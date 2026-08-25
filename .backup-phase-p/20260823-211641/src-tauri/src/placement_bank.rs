use crate::placement::{
    CefrBand, PlacementOptionDto, PlacementQuestionDto, PlacementSkill, PlacementSpeakingPromptDto,
    PLACEMENT_QUESTION_BANK_VERSION, PLACEMENT_SPEAKING_PROMPT_VERSION,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const BANK_JSON: &str = include_str!("../resources/placement/placement_bank_v1.json");

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementOption {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementQuestion {
    pub id: String,
    pub skill: PlacementSkill,
    pub cefr_band: CefrBand,
    pub prompt: String,
    pub options: Vec<PlacementOption>,
    pub correct_option_id: String,
    #[serde(default)]
    pub passage_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlacementPassage {
    pub id: String,
    pub band: CefrBand,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSpeakingPrompt {
    pub id: String,
    pub sequence_index: u32,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementBank {
    pub version: u32,
    pub speaking_prompt_version: u32,
    pub passages: Vec<PlacementPassage>,
    pub speaking_prompts: Vec<PlacementSpeakingPrompt>,
    pub questions: Vec<PlacementQuestion>,
}

impl PlacementBank {
    pub fn load() -> Result<Self, String> {
        let bank: Self = serde_json::from_str(BANK_JSON)
            .map_err(|e| format!("Placement question bank is invalid JSON: {e}"))?;
        bank.validate()?;
        Ok(bank)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != PLACEMENT_QUESTION_BANK_VERSION {
            return Err("Unsupported placement question bank version.".into());
        }
        if self.speaking_prompt_version != PLACEMENT_SPEAKING_PROMPT_VERSION {
            return Err("Unsupported placement speaking prompt version.".into());
        }
        let mut question_ids = HashSet::new();
        let passage_map: HashMap<_, _> = self.passages.iter().map(|p| (p.id.as_str(), p)).collect();
        if passage_map.len() != self.passages.len()
            || self.passages.iter().any(|p| p.text.trim().is_empty())
        {
            return Err("Placement passages must have unique IDs and non-empty text.".into());
        }
        for band in [
            CefrBand::A1,
            CefrBand::A2,
            CefrBand::B1,
            CefrBand::B2,
            CefrBand::C1,
            CefrBand::C2,
        ] {
            let passages = self.passages.iter().filter(|p| p.band == band).count();
            if passages != 1 {
                return Err(format!(
                    "Reading band {} must have exactly one passage.",
                    band.as_str()
                ));
            }
            for skill in PlacementSkill::ALL {
                let count = self
                    .questions
                    .iter()
                    .filter(|q| q.skill == skill && q.cefr_band == band)
                    .count();
                if count != 3 {
                    return Err(format!(
                        "{} {} must contain exactly 3 questions.",
                        skill.as_str(),
                        band.as_str()
                    ));
                }
            }
        }
        for question in &self.questions {
            if !question_ids.insert(question.id.as_str())
                || question.prompt.trim().is_empty()
                || question.options.len() < 2
            {
                return Err(format!("Invalid placement question {}.", question.id));
            }
            let option_ids: HashSet<_> = question.options.iter().map(|o| o.id.as_str()).collect();
            if option_ids.len() != question.options.len()
                || question.options.iter().any(|o| o.text.trim().is_empty())
                || !option_ids.contains(question.correct_option_id.as_str())
            {
                return Err(format!(
                    "Invalid options or answer key for {}.",
                    question.id
                ));
            }
            match (question.skill, question.passage_id.as_deref()) {
                (PlacementSkill::Reading, Some(id))
                    if passage_map
                        .get(id)
                        .is_some_and(|p| p.band == question.cefr_band) => {}
                (PlacementSkill::Reading, _) => {
                    return Err(format!(
                        "Reading question {} has an invalid passage.",
                        question.id
                    ))
                }
                (_, None) => {}
                (_, Some(_)) => {
                    return Err(format!(
                        "Non-reading question {} cannot have a passage.",
                        question.id
                    ))
                }
            }
        }
        let prompt_ids: HashSet<_> = self
            .speaking_prompts
            .iter()
            .map(|p| p.id.as_str())
            .collect();
        let indexes: HashSet<_> = self
            .speaking_prompts
            .iter()
            .map(|p| p.sequence_index)
            .collect();
        if self.speaking_prompts.len() != 3
            || prompt_ids.len() != 3
            || indexes != HashSet::from([0, 1, 2])
            || self
                .speaking_prompts
                .iter()
                .any(|p| p.text.trim().is_empty())
        {
            return Err("Placement requires three unique speaking prompts.".into());
        }
        Ok(())
    }

    pub fn question(&self, id: &str) -> Option<&PlacementQuestion> {
        self.questions.iter().find(|q| q.id == id)
    }
    pub fn question_at(
        &self,
        skill: PlacementSkill,
        band: CefrBand,
        index: usize,
    ) -> Option<&PlacementQuestion> {
        self.questions
            .iter()
            .filter(|q| q.skill == skill && q.cefr_band == band)
            .nth(index)
    }
    pub fn public_question(&self, question: &PlacementQuestion) -> PlacementQuestionDto {
        PlacementQuestionDto {
            question_id: question.id.clone(),
            skill: question.skill,
            prompt: question.prompt.clone(),
            options: question
                .options
                .iter()
                .map(|o| PlacementOptionDto {
                    id: o.id.clone(),
                    text: o.text.clone(),
                })
                .collect(),
            passage: question
                .passage_id
                .as_deref()
                .and_then(|id| self.passages.iter().find(|p| p.id == id))
                .map(|p| p.text.clone()),
        }
    }
    pub fn speaking_prompt(&self, sequence: usize) -> Option<&PlacementSpeakingPrompt> {
        self.speaking_prompts
            .iter()
            .find(|p| p.sequence_index as usize == sequence)
    }
    pub fn public_speaking_prompt(
        &self,
        prompt: &PlacementSpeakingPrompt,
    ) -> PlacementSpeakingPromptDto {
        PlacementSpeakingPromptDto {
            prompt_id: prompt.id.clone(),
            prompt_version: self.speaking_prompt_version,
            sequence_index: prompt.sequence_index,
            prompt: prompt.text.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bank_has_exact_complete_structure() {
        let bank = PlacementBank::load().unwrap();
        assert_eq!(bank.questions.len(), 54);
        for skill in PlacementSkill::ALL {
            assert_eq!(
                bank.questions.iter().filter(|q| q.skill == skill).count(),
                18
            );
        }
        assert_eq!(bank.passages.len(), 6);
        assert_eq!(bank.speaking_prompts.len(), 3);
    }
    #[test]
    fn public_question_never_serializes_answer_key_or_band() {
        let bank = PlacementBank::load().unwrap();
        let value = serde_json::to_string(&bank.public_question(&bank.questions[0])).unwrap();
        assert!(!value.contains("correctOptionId"));
        assert!(!value.contains("isCorrect"));
        assert!(!value.contains("cefrBand"));
    }
}
