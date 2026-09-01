use crate::sha256;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

pub const TOEIC_ITEM_BANK_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToeicSection {
    Listening,
    Reading,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToeicPart {
    Part1Photograph,
    Part2QuestionResponse,
    Part3Conversation,
    Part4Talk,
    Part5IncompleteSentence,
    Part6TextCompletion,
    Part7ReadingComprehension,
}

impl ToeicPart {
    pub fn runtime_available(self) -> bool {
        self == Self::Part1Photograph
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Part1Photograph => "part1_photograph",
            Self::Part2QuestionResponse => "part2_question_response",
            Self::Part3Conversation => "part3_conversation",
            Self::Part4Talk => "part4_talk",
            Self::Part5IncompleteSentence => "part5_incomplete_sentence",
            Self::Part6TextCompletion => "part6_text_completion",
            Self::Part7ReadingComprehension => "part7_reading_comprehension",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToeicDifficulty {
    Easy,
    Medium,
    Hard,
}

impl ToeicDifficulty {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationState {
    Draft,
    Published,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToeicStatement {
    pub choice: String,
    pub text: String,
    pub distractor_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToeicAsset {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToeicItem {
    pub item_id: String,
    pub item_version: u32,
    pub publication_state: PublicationState,
    pub section: ToeicSection,
    pub part: ToeicPart,
    pub difficulty: ToeicDifficulty,
    pub skill_tags: Vec<String>,
    pub image: ToeicAsset,
    pub statements: Vec<ToeicStatement>,
    pub correct_answer: String,
    pub correct_explanation: String,
    pub distractor_explanations: BTreeMap<String, String>,
    pub language_focus: Vec<String>,
    pub useful_vocabulary: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToeicFormItemRef {
    pub item_id: String,
    pub item_version: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToeicForm {
    pub form_id: String,
    pub form_version: u32,
    pub publication_state: PublicationState,
    pub section: ToeicSection,
    pub part: ToeicPart,
    pub items: Vec<ToeicFormItemRef>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawBank {
    bank_schema_version: u32,
    bank_id: String,
    items: Vec<ToeicItem>,
    forms: Vec<ToeicForm>,
}

#[derive(Clone)]
pub struct ToeicItemBank {
    root: PathBuf,
    bank_id: String,
    items: BTreeMap<(String, u32), ToeicItem>,
    forms: BTreeMap<(String, u32), ToeicForm>,
}

impl ToeicItemBank {
    pub fn load(root: PathBuf) -> Result<Self, String> {
        let manifest = root.join("bank.json");
        let metadata =
            fs::metadata(&manifest).map_err(|_| "TOEIC bank.json is missing.".to_owned())?;
        if metadata.len() > 2 * 1024 * 1024 {
            return Err("TOEIC bank.json exceeds 2 MiB.".into());
        }
        let bank: RawBank = serde_json::from_slice(
            &fs::read(&manifest).map_err(|_| "TOEIC bank.json could not be read.".to_owned())?,
        )
        .map_err(|error| format!("TOEIC bank.json is invalid: {error}"))?;
        validate_bank(&root, &bank)?;
        Ok(Self {
            root,
            bank_id: bank.bank_id,
            items: bank
                .items
                .into_iter()
                .map(|item| ((item.item_id.clone(), item.item_version), item))
                .collect(),
            forms: bank
                .forms
                .into_iter()
                .filter(|form| form.publication_state == PublicationState::Published)
                .map(|form| ((form.form_id.clone(), form.form_version), form))
                .collect(),
        })
    }

    pub fn bank_id(&self) -> &str {
        &self.bank_id
    }
    pub fn published_forms(&self) -> Vec<ToeicForm> {
        self.forms.values().cloned().collect()
    }
    pub fn form(&self, id: &str, version: u32) -> Option<ToeicForm> {
        self.forms.get(&(id.to_owned(), version)).cloned()
    }
    pub fn item(&self, id: &str, version: u32) -> Option<ToeicItem> {
        self.items.get(&(id.to_owned(), version)).cloned()
    }
    pub fn image_bytes(&self, item: &ToeicItem) -> Result<Vec<u8>, String> {
        let path = safe_asset_path(&self.root, &item.image.path)?;
        fs::read(path).map_err(|_| "TOEIC photograph is unavailable.".to_owned())
    }
    #[cfg(test)]
    pub fn item_count(&self) -> usize {
        self.items
            .values()
            .filter(|item| item.publication_state == PublicationState::Published)
            .count()
    }
}

fn validate_bank(root: &Path, bank: &RawBank) -> Result<(), String> {
    if bank.bank_schema_version != TOEIC_ITEM_BANK_SCHEMA_VERSION {
        return Err("Unsupported TOEIC item bank schema.".into());
    }
    bounded(&bank.bank_id, 1, 80, "bankId")?;
    if bank.items.is_empty() {
        return Err("TOEIC item bank is empty.".into());
    }
    let allowed_tags = BTreeSet::from([
        "people_actions",
        "object_actions",
        "object_state",
        "location",
        "prepositions",
        "present_progressive",
        "passive_description",
        "spatial_relationships",
        "workplace_objects",
        "transportation",
        "indoor_scene",
        "outdoor_scene",
        "similar_word_distractor",
        "wrong_action_distractor",
        "wrong_location_distractor",
    ]);
    let allowed_distractors = BTreeSet::from([
        "wrong_action",
        "wrong_person",
        "wrong_location",
        "wrong_state",
        "related_vocabulary",
    ]);
    let mut item_keys = BTreeSet::new();
    for item in &bank.items {
        validate_id(&item.item_id, "itemId")?;
        if item.item_version == 0 || !item_keys.insert((item.item_id.clone(), item.item_version)) {
            return Err("TOEIC item id/version must be unique and positive.".into());
        }
        if item.section != ToeicSection::Listening || item.part != ToeicPart::Part1Photograph {
            return Err("Phase 1 bank may publish only Listening Part 1 items.".into());
        }
        if item.skill_tags.is_empty()
            || item.skill_tags.len() > 8
            || item
                .skill_tags
                .iter()
                .any(|tag| !allowed_tags.contains(tag.as_str()))
        {
            return Err(format!("{} has invalid skill tags.", item.item_id));
        }
        if item.statements.len() != 4 {
            return Err(format!(
                "{} must contain exactly four statements.",
                item.item_id
            ));
        }
        let mut choices = BTreeSet::new();
        let mut texts = BTreeSet::new();
        for statement in &item.statements {
            if !matches!(statement.choice.as_str(), "A" | "B" | "C" | "D")
                || !choices.insert(statement.choice.clone())
            {
                return Err(format!("{} must have unique A-D choices.", item.item_id));
            }
            bounded(&statement.text, 3, 240, "statement")?;
            if !texts.insert(statement.text.trim().to_lowercase()) {
                return Err(format!("{} has duplicate statements.", item.item_id));
            }
            if statement.choice == item.correct_answer {
                if statement.distractor_type.is_some() {
                    return Err(format!(
                        "{} correct choice cannot have distractor metadata.",
                        item.item_id
                    ));
                }
            } else if !statement
                .distractor_type
                .as_deref()
                .is_some_and(|value| allowed_distractors.contains(value))
            {
                return Err(format!("{} has an invalid distractor type.", item.item_id));
            }
        }
        if choices
            != BTreeSet::from([
                "A".to_owned(),
                "B".to_owned(),
                "C".to_owned(),
                "D".to_owned(),
            ])
            || !choices.contains(&item.correct_answer)
        {
            return Err(format!("{} has an invalid answer key.", item.item_id));
        }
        bounded(&item.correct_explanation, 20, 600, "correctExplanation")?;
        let expected_distractors = choices
            .iter()
            .filter(|choice| **choice != item.correct_answer)
            .cloned()
            .collect::<BTreeSet<_>>();
        if item
            .distractor_explanations
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != expected_distractors
            || item
                .distractor_explanations
                .values()
                .any(|value| bounded(value, 20, 600, "distractorExplanation").is_err())
        {
            return Err(format!(
                "{} must explain every distractor exactly once.",
                item.item_id
            ));
        }
        if item.language_focus.is_empty()
            || item.language_focus.len() > 2
            || item
                .language_focus
                .iter()
                .any(|value| bounded(value, 3, 240, "languageFocus").is_err())
        {
            return Err(format!("{} has invalid language focus.", item.item_id));
        }
        if item.useful_vocabulary.len() > 4
            || item
                .useful_vocabulary
                .iter()
                .any(|value| bounded(value, 3, 120, "usefulVocabulary").is_err())
        {
            return Err(format!("{} has invalid useful vocabulary.", item.item_id));
        }
        let image_path = safe_asset_path(root, &item.image.path)?;
        let metadata =
            fs::metadata(&image_path).map_err(|_| format!("{} image is missing.", item.item_id))?;
        if !metadata.is_file()
            || metadata.len() < 50_000
            || metadata.len() > 10 * 1024 * 1024
            || image_path.extension().and_then(|value| value.to_str()) != Some("png")
        {
            return Err(format!(
                "{} image is not a valid production PNG.",
                item.item_id
            ));
        }
        if sha256::file(&image_path).map_err(|_| "Could not hash TOEIC image.".to_owned())?
            != item.image.sha256.to_uppercase()
        {
            return Err(format!("{} image hash mismatch.", item.item_id));
        }
    }
    let mut form_keys = BTreeSet::new();
    for form in &bank.forms {
        validate_id(&form.form_id, "formId")?;
        if form.form_version == 0 || !form_keys.insert((form.form_id.clone(), form.form_version)) {
            return Err("TOEIC form id/version must be unique and positive.".into());
        }
        if form.section != ToeicSection::Listening
            || form.part != ToeicPart::Part1Photograph
            || form.items.len() != 6
        {
            return Err(format!(
                "{} must contain exactly six Part 1 items.",
                form.form_id
            ));
        }
        let mut refs = BTreeSet::new();
        let mut difficulties = BTreeMap::<ToeicDifficulty, usize>::new();
        for reference in &form.items {
            if !refs.insert((reference.item_id.clone(), reference.item_version)) {
                return Err(format!("{} contains a duplicate item.", form.form_id));
            }
            let item = bank
                .items
                .iter()
                .find(|item| {
                    item.item_id == reference.item_id && item.item_version == reference.item_version
                })
                .ok_or_else(|| format!("{} references a missing item.", form.form_id))?;
            if item.publication_state != PublicationState::Published {
                return Err(format!("{} references a non-published item.", form.form_id));
            }
            *difficulties.entry(item.difficulty).or_default() += 1;
        }
        if form.publication_state == PublicationState::Published
            && difficulties
                != BTreeMap::from([
                    (ToeicDifficulty::Easy, 2),
                    (ToeicDifficulty::Medium, 2),
                    (ToeicDifficulty::Hard, 2),
                ])
        {
            return Err(format!(
                "{} must contain two items at each difficulty.",
                form.form_id
            ));
        }
    }
    Ok(())
}

fn safe_asset_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("TOEIC asset path is unsafe.".into());
    }
    let full = root.join(path);
    if fs::symlink_metadata(&full)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err("TOEIC asset symlinks are not allowed.".into());
    }
    Ok(full)
}
fn validate_id(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!("{label} must be a lowercase slug."));
    }
    Ok(())
}
fn bounded(value: &str, min: usize, max: usize, label: &str) -> Result<(), String> {
    let count = value.trim().chars().count();
    if count < min || count > max {
        return Err(format!("{label} length is invalid."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_bank_has_three_valid_six_item_forms_and_eighteen_original_assets() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/toeic/item-bank-v1");
        let bank = ToeicItemBank::load(root).unwrap();
        assert_eq!(bank.item_count(), 18);
        assert_eq!(bank.published_forms().len(), 3);
        assert!(bank
            .published_forms()
            .iter()
            .all(|form| form.items.len() == 6));
    }

    #[test]
    fn part_runtime_availability_is_future_proof_but_only_part_one_is_enabled() {
        assert!(ToeicPart::Part1Photograph.runtime_available());
        assert!(!ToeicPart::Part2QuestionResponse.runtime_available());
        assert!(!ToeicPart::Part7ReadingComprehension.runtime_available());
    }

    #[test]
    fn validator_rejects_duplicate_statements_unsafe_assets_and_incomplete_forms() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/toeic/item-bank-v1");
        let raw = fs::read(root.join("bank.json")).unwrap();
        let bank: RawBank = serde_json::from_slice(&raw).unwrap();
        let mut duplicate = bank.clone();
        duplicate.items[0].statements[1].text = duplicate.items[0].statements[0].text.clone();
        assert!(validate_bank(&root, &duplicate)
            .unwrap_err()
            .contains("duplicate statements"));
        let mut unsafe_asset = bank.clone();
        unsafe_asset.items[0].image.path = "../secret.png".into();
        assert!(validate_bank(&root, &unsafe_asset)
            .unwrap_err()
            .contains("unsafe"));
        let mut incomplete = bank;
        incomplete.forms[0].items.pop();
        assert!(validate_bank(&root, &incomplete)
            .unwrap_err()
            .contains("exactly six"));
    }
}
