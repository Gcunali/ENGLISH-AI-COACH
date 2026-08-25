import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(r"C:\ENGLISH AI COACH")
LESSONS_ROOT = ROOT / "src-tauri" / "resources" / "interactive-lessons"
CURRICULUM_PATH = ROOT / "src-tauri" / "resources" / "curriculum" / "english-core" / "curriculum.json"
DOCS_ROOT = ROOT / "docs" / "content"

LEVEL_META = {
    "B1": {"title": "Intermediate", "description": "Communicate independently in familiar situations, explain experiences, and solve practical problems.", "minutes": 36, "vocab": 10, "listen": 5, "repeat": 6, "speak": 5, "exercises": 10, "turns": (6, 8, 12), "theory": (250, 450)},
    "B2": {"title": "Upper Intermediate", "description": "Structure arguments, negotiate solutions, and communicate confidently in complex situations.", "minutes": 42, "vocab": 11, "listen": 6, "repeat": 7, "speak": 6, "exercises": 11, "turns": (7, 10, 14), "theory": (300, 550)},
    "C1": {"title": "Advanced", "description": "Communicate with precision, nuance, register control, and sustained argumentation.", "minutes": 48, "vocab": 12, "listen": 7, "repeat": 8, "speak": 7, "exercises": 12, "turns": (8, 11, 16), "theory": (350, 600)},
    "C2": {"title": "Proficiency Content", "description": "Use English flexibly and precisely across subtle, high-complexity interactions.", "minutes": 52, "vocab": 12, "listen": 8, "repeat": 8, "speak": 8, "exercises": 12, "turns": (8, 12, 18), "theory": (350, 650)},
}


def unit(level, title, lessons, focuses, language):
    return {"level": level, "title": title, "lessons": lessons.split("|"), "focuses": focuses.split("|"), "language": language.split("|")}


UNITS = [
unit("B1", "Experiences & Storytelling", "Experiences & Events|Present Perfect or Past Simple?|For & Since|Building a Story|Something Unexpected Happened|Tell Your Story", "present perfect for experience|present perfect versus finished past|for and since with continuing situations|narrative sequence and linkers|past continuous interrupted by past simple|integrated experience and narrative review", "memorable experience|take part in|turning point|so far|recently|for several years|since I was a child|at first|after a while|all of a sudden|in the end|look back on|It turned out that|What happened next?|The experience taught me that"),
unit("B1", "Plans, Possibilities & Decisions", "Plans & Intentions|Future Choices|If This Happens...|What Would You Do?|Possibilities & Decisions|Making a Difficult Decision", "going to and present continuous for plans|will, going to, and present continuous choices|first conditional|basic second conditional|may, might, could, and decision language|integrated future and conditional review", "intend to|arrange to|weigh the options|make up my mind|likely outcome|possible consequence|backup plan|on the one hand|on the other hand|If everything goes well|If I were in that situation|I might consider|The safest option would be|It depends on|reach a decision"),
unit("B1", "Work & Study", "Work & Career|Skills & Responsibilities|Gerund or Infinitive?|People Who Work With Me|Goals, Projects & Challenges|Career Conversation", "present forms for roles and routines|can, be responsible for, and have to|common gerund and infinitive patterns|defining relative clauses|project language and goal structures|integrated work and study review", "career path|job role|be responsible for|meet a deadline|workload|practical skill|look forward to|decide to|team member|someone who|long-term goal|take on a challenge|make progress|deal with a setback|apply for a position"),
unit("B1", "Communication & Relationships", "Talking About People|What Did They Say?|Say or Tell?|Asking Indirect Questions|Problems & Misunderstandings|Resolving a Misunderstanding", "relative clauses for people|introductory reported speech|say versus tell patterns|indirect question word order|clarifying and explaining misunderstandings|integrated relationship repair review", "get along with|have in common|reliable|considerate|They said that|She told me to|Could you tell me whether|Do you know what|misunderstanding|mixed message|clear things up|I may have misunderstood|What I meant was|see another point of view|work things out"),
unit("B1", "Travel & Problem Solving", "Travel Experiences|Delays & Changes|Asking for Help Politely|Explaining a Problem|Finding a Solution|Travel Problem Mission", "present perfect and past travel detail|passive updates and changed arrangements|polite modal requests|problem, cause, and effect language|suggestions and solution language|integrated travel problem review", "travel abroad|local customs|delayed connection|rebook a flight|change of plan|Would you mind helping me?|Could you let me know|The issue is that|due to|as a result|available alternative|sort out|make a complaint|offer a refund|reach the destination"),
unit("B1", "Health, Habits & Lifestyle", "Healthy Habits|Changes Over Time|I Used To...|Giving Better Advice|Lifestyle Choices|Changing a Habit", "present simple and habit collocations|comparative change language|used to versus present habits|should, ought to, and had better|choice, reason, and consequence|integrated habit-change review", "balanced diet|sleep routine|stay active|cut down on|gradual change|over time|used to|get used to|You ought to|You'd better|realistic goal|healthy choice|keep track of|break a habit|stick to a plan"),
unit("B1", "Technology, Media & Opinions", "Technology in Our Lives|News & Media|Active or Passive?|Expressing Opinions|Advantages & Disadvantages|Technology Discussion", "technology habits and present forms|news vocabulary and source language|introductory active and passive voice|opinion and supporting reason|balanced advantage and disadvantage linkers|integrated technology discussion review", "digital tool|stay connected|privacy setting|reliable source|headline|report an event|is used for|was created by|From my perspective|The main reason is|major advantage|potential drawback|whereas|on balance|raise a concern"),
unit("B1", "Independent Communication", "Explaining Your Point|Solving Everyday Problems|Telling Detailed Stories|Making Suggestions|Discussing Different Options|B1 Real-Life Mission", "linking reasons and examples|problem-solution sequence|integrated narrative tenses|suggestion modals and phrases|comparison and conditional choices|integrated B1 communication review", "make a point|give an example|in other words|practical problem|possible solution|step by step|set the scene|unexpected event|Why don't we|We could always|compare the options|take cost into account|come to an agreement|explain clearly|handle the situation"),

unit("B2", "Advanced Storytelling", "Telling Better Stories|Past Perfect|Setting the Scene|Used To & Would|Building Suspense|Storytelling Mission", "narrative tense control|past perfect for earlier events|descriptive background and viewpoint|used to and would for repeated past actions|suspense, pacing, and delayed information|integrated advanced storytelling review", "compelling account|vivid detail|sequence of events|had already happened|by the time|set the scene|in the distance|would often|used to be|little did I know|build suspense|keep someone guessing|eventually reveal|unexpected twist|with hindsight"),
unit("B2", "Hypothetical Situations", "Imagining Different Situations|Second Conditional in Depth|Third Conditional|Wish & If Only|Regrets & Alternatives|What Would Have Happened?", "hypothetical framing|second conditional nuance|third conditional|wish and if only patterns|past regret and alternative outcomes|integrated hypothetical review", "hypothetical scenario|imagine that|provided that|were to happen|If I had known|would have changed|I wish I could|If only we had|regret doing|missed opportunity|alternative outcome|in retrospect|otherwise|under different circumstances|There is no way of knowing"),
unit("B2", "Deduction, Possibility & Certainty", "How Certain Are You?|Must, Might & Can't|What Might Have Happened?|Should Have|Speculating About Events|Mystery Discussion", "degrees of certainty|present modal deduction|past modal deduction|should have for criticism and expectation|evidence-based speculation|integrated deduction review", "almost certain|highly likely|reasonable possibility|must be|might be|can't be|may have overlooked|could have happened|should have checked|unexpected evidence|draw a conclusion|rule out|point to|based on the evidence|plausible explanation"),
unit("B2", "Passive & Reporting", "The Passive Voice|Passive Across Tenses|Reporting Information|Reporting Verbs|Have Something Done|News & Reporting Mission", "passive focus and agent choice|passive across common tenses|reported statements and questions|reporting verb patterns|basic causative have|integrated reporting review", "be carried out|was announced|has been confirmed|is expected to|according to|official statement|reportedly|claim that|deny doing|warn someone to|encourage someone to|have it repaired|independent source|public response|verify the information"),
unit("B2", "Professional Communication", "Professional Communication|Meetings & Discussions|Presenting an Idea|Agreeing & Disagreeing Professionally|Negotiating Solutions|Workplace Challenge", "professional register and clarity|meeting management language|structured presentation language|diplomatic agreement and disagreement|negotiation and compromise|integrated workplace communication review", "clarify the objective|key priority|action item|bring up a point|keep to the agenda|outline a proposal|main benefit|I take your point|I have some reservations|find common ground|workable compromise|meet halfway|resource constraint|follow-up action|reach an agreement"),
unit("B2", "Academic & Structured Communication", "Organizing Your Ideas|Cause & Effect|Contrast & Comparison|Giving Examples|Hedging Your Opinion|Explaining a Complex Idea", "structured discourse markers|cause and effect chains|contrast and comparison structures|example and illustration language|hedging claims and opinions|integrated complex explanation review", "central claim|supporting point|logical sequence|lead to|stem from|consequently|in contrast|by comparison|a clear example is|to illustrate|tend to|appears to|to some extent|underlying factor|draw a distinction"),
unit("B2", "Society, Media & Debate", "Stronger Opinions|Supporting an Argument|Understanding Another View|Counterarguments|Media & Society|Structured Debate", "qualified strong opinions|claim, reason, and evidence|paraphrasing another viewpoint|concession and counterargument|media influence discourse|integrated structured debate review", "firmly believe|contentious issue|support a claim|relevant evidence|underlying assumption|from their perspective|a fair point|concede that|counterargument|nevertheless|media coverage|public perception|bias|weigh the evidence|reach a balanced conclusion"),
unit("B2", "Advanced Real-Life Communication", "Handling Complex Problems|Persuading Someone|Explaining Difficult Ideas|Managing Disagreement|Discussing Alternatives|B2 Real-Life Mission", "multi-step problem explanation|persuasive reason and benefit|clarification and analogy|de-escalating disagreement|evaluating alternatives|integrated B2 communication review", "complex issue|root cause|practical constraint|convincing case|mutual benefit|address a concern|put it another way|use an analogy|keep the discussion constructive|acknowledge a concern|viable alternative|trade-off|preferred course of action|contingency plan|resolve the matter"),

unit("C1", "Precision & Nuance", "Saying Exactly What You Mean|Degrees of Certainty|Subtle Differences in Meaning|Choosing the Right Word|Emphasis & Qualification|Precision Mission", "precise reformulation|nuanced epistemic modality|near-synonym distinctions|lexical choice by context and connotation|controlled emphasis and qualification|integrated precision review", "to be precise|more specifically|broadly speaking|all but certain|conceivable|a remote possibility|subtle distinction|carry a connotation|context-dependent|apt description|precise wording|not so much X as Y|particularly significant|subject to qualification|avoid overgeneralizing"),
unit("C1", "Advanced Grammar in Context", "Advanced Conditionals|Mixed Conditionals|Advanced Relative Structures|Participle Clauses|Emphasis & Inversion|Grammar in Real Communication", "advanced conditional alternatives|mixed-time conditionals|non-defining and reduced relative structures|participle clauses for information flow|inversion and cleft emphasis|integrated advanced grammar review", "assuming that|on condition that|but for|Had it not been for|If I were better prepared|which in turn|many of whom|the extent to which|having considered|given the circumstances|rarely have I|what matters most is|under no circumstances|grammatical emphasis|information flow"),
unit("C1", "Advanced Professional English", "Leading a Discussion|Presenting Complex Information|Diplomatic Disagreement|Negotiating Strategically|Managing Difficult Conversations|Professional Leadership Mission", "facilitation and turn management|layered presentation structure|diplomatic challenge and mitigation|strategic negotiation|sensitive conversation management|integrated professional leadership review", "frame the discussion|invite contributions|draw together|key implication|walk through the findings|a degree of uncertainty|I wonder whether|That may overlook|strategic priority|negotiating position|non-negotiable constraint|address tension|reframe the issue|preserve trust|agree on next steps"),
unit("C1", "Academic Communication", "Building an Academic Argument|Evidence & Interpretation|Hedging & Caution|Synthesizing Information|Explaining Research|Academic Discussion Mission", "thesis and argument architecture|evidence versus interpretation|academic hedging|source synthesis and relationship|research explanation and limitations|integrated academic discussion review", "advance an argument|central premise|line of reasoning|empirical evidence|plausible interpretation|cannot establish|arguably|may indicate|taken together|converging evidence|contradictory finding|research design|methodological limitation|warrant further study|qualified conclusion"),
unit("C1", "Persuasion & Rhetoric", "Persuasive Language|Framing an Argument|Challenging an Idea Politely|Concession & Counterargument|Influencing an Audience|Persuasion Mission", "ethical persuasive appeals|argument framing|polite rigorous challenge|concession and rebuttal|audience adaptation|integrated persuasion review", "compelling reason|appeal to values|credible evidence|frame the issue|shift the focus|underlying premise|May I challenge|I am not entirely convinced|admittedly|even so|anticipate an objection|audience concern|call to action|measured tone|persuasive impact"),
unit("C1", "Idiomatic & Natural English", "Natural Collocations|Advanced Phrasal Verbs|Idiomatic Expressions in Context|Formal vs Conversational English|Sounding More Natural|Natural Communication Mission", "high-frequency advanced collocations|advanced phrasal verbs in context|transparent idiomatic meaning|register-sensitive alternatives|discourse rhythm and natural chunks|integrated natural communication review", "reach a consensus|raise awareness|pose a challenge|phase out|follow through on|come across as|read between the lines|a grey area|the bigger picture|conduct an investigation|look into the matter|to be honest|having said that|natural turn-taking|strike the right tone"),
unit("C1", "Society, Ideas & Complex Topics", "Discussing Social Change|Technology & Ethics|Education & Opportunity|Work & Society|Different Perspectives|Advanced Debate", "change, trend, and impact discourse|ethical reasoning and trade-offs|opportunity and inequality discourse|labor and social impact|perspective synthesis|integrated advanced debate review", "social shift|long-term trend|uneven impact|ethical implication|informed consent|unintended consequence|equal access|structural barrier|social mobility|changing workforce|public interest|competing perspective|shared concern|broader context|nuanced position"),
unit("C1", "Flexible Advanced Communication", "Explaining Abstract Ideas|Adapting Your Register|Responding Under Pressure|Reformulating Your Message|Managing Complex Interaction|C1 Real-Life Mission", "analogy and abstraction|register adaptation|structured spontaneous response|advanced reformulation|interaction repair and redirection|integrated C1 communication review", "abstract concept|concrete analogy|underlying principle|adapt the register|level of formality|audience expectation|think on your feet|initial response|let me qualify that|to put it differently|what I am getting at|manage the floor|redirect the discussion|resolve ambiguity|sustain the interaction"),

unit("C2", "Meaning, Ambiguity & Precision", "Fine Shades of Meaning|Managing Ambiguity|Implication & Inference|Precise Reformulation|Saying More With Less|Precision & Meaning Mission", "fine-grained lexical distinctions|productive ambiguity management|pragmatic implication and inference|meaning-preserving reformulation|precision through concision|integrated meaning and precision review", "fine distinction|semantic range|pragmatic force|deliberately ambiguous|open to interpretation|resolve the ambiguity|unstated implication|reasonable inference|read into|meaning-preserving|recast the point|economy of expression|cut through the detail|precisely put|retain the nuance"),
unit("C2", "Mastering Register", "Formal, Neutral & Informal|Diplomatic Language|Professional Nuance|Academic Register|Switching Register Naturally|Register Control Mission", "three-way register calibration|highly diplomatic mitigation|professional subtext and nuance|dense but clear academic register|real-time register switching|integrated register control review", "register continuum|stylistic choice|plain equivalent|with due respect|I would be hesitant to|tactful reservation|professional subtext|strategic understatement|conceptual framework|analytical rigor|disciplinary convention|shift the tone|code-switch stylistically|audience calibration|maintain authenticity"),
unit("C2", "Complex Argumentation", "Building Multi-Layered Arguments|Evaluating Assumptions|Challenging Evidence|Balancing Perspectives|Reframing an Argument|Advanced Argument Mission", "multi-layered argument architecture|assumption analysis|evidential challenge|weighted perspective synthesis|strategic reframing|integrated complex argument review", "subsidiary claim|interdependent reason|logical consequence|tacit assumption|questionable premise|scope condition|evidential threshold|alternative explanation|countervailing consideration|relative weight|reconcile two views|reframe the question|change the terms of debate|robust conclusion|remain provisional"),
unit("C2", "Rhetoric & Persuasion", "Rhetorical Choices|Strategic Emphasis|Persuasive Framing|Subtle Agreement & Disagreement|Influencing Without Overstating|Rhetorical Mission", "rhetorical choice and effect|strategic foregrounding|persuasive frame selection|layered alignment and dissent|calibrated influence|integrated rhetorical control review", "rhetorical effect|deliberate choice|shape interpretation|foreground a concern|strategic repetition|measured emphasis|persuasive frame|value-laden term|qualified agreement|quiet reservation|nudge the audience|avoid overclaiming|credible restraint|leave room for doubt|rhetorical control"),
unit("C2", "Complex Professional Communication", "Executive Communication|High-Stakes Discussions|Complex Negotiation|Handling Sensitive Disagreement|Strategic Presentations|Executive Communication Mission", "concise executive synthesis|high-stakes risk communication|multi-variable negotiation|face-sensitive disagreement|strategic narrative in presentations|integrated executive communication review", "executive summary|decision-critical|material risk|high-stakes consequence|escalation path|shared accountability|negotiating leverage|package an agreement|sensitive concern|preserve working relations|strategic narrative|decision point|scenario planning|state a recommendation|secure alignment"),
unit("C2", "Advanced Academic & Intellectual Discussion", "Synthesizing Complex Ideas|Critiquing an Argument|Discussing Method & Evidence|Expressing Intellectual Caution|Developing a Complex Position|Academic Synthesis Mission", "complex cross-source synthesis|principled argument critique|method-evidence alignment|intellectual caution and scope|evolving multi-part position|integrated academic synthesis review", "conceptual synthesis|interdisciplinary link|apparent contradiction|internal coherence|explanatory power|analytical weakness|methodological fit|quality of evidence|epistemic limit|withhold judgment|tentative proposition|evolving position|integrate the objection|intellectual honesty|defensible synthesis"),
unit("C2", "Idiomaticity & Stylistic Control", "Advanced Collocations|Idioms With Precision|Metaphorical Language|Style & Tone|Concision & Elegance|Stylistic Control Mission", "precise advanced collocation|idiom constrained by context|controlled conceptual metaphor|stylistic and tonal modulation|elegant compression|integrated stylistic control review", "deeply entrenched|glaring discrepancy|draw a sharp distinction|move the goalposts|walk a fine line|lose sight of|conceptual metaphor|frame as a journey|stylistic texture|tonal shift|deliberate informality|remove redundancy|compress the argument|elegant phrasing|stylistic consistency"),
unit("C2", "Mastery of Flexible Communication", "Handling Unexpected Topics|Explaining Subtle Differences|Adapting in Real Time|Managing Complex Dialogue|Integrating Multiple Perspectives|C2 Real-Life Mission", "rapid framing of unfamiliar topics|spontaneous fine distinction|real-time strategy adaptation|multi-party dialogue management|complex perspective integration|integrated C2 communication review", "unfamiliar angle|provisional frame|draw on analogy|subtle difference|material distinction|contextual nuance|adapt on the fly|revise the approach|track several threads|surface an assumption|mediate the exchange|integrate perspectives|preserve disagreement|coherent synthesis|flexible response"),
]


def slug(value):
    value = value.lower().replace("&", "and")
    value = re.sub(r"[^a-z0-9]+", "-", value).strip("-")
    return value


def level_units(level):
    return [u for u in UNITS if u["level"] == level]


def language_slice(unit_data, lesson_index, count):
    bank = unit_data["language"]
    start = lesson_index * 2
    return [bank[(start + i) % len(bank)] for i in range(count)]


def sentence(chunk, unit_title, index):
    c = chunk.strip()
    if c[:1].isupper() or c.startswith(("If ", "Had ", "May ", "Could ", "Would ", "Do ", "What ", "Why ", "There ", "That ", "Little ", "Rarely ", "Under ")):
        return c if c.endswith((".", "?", "!")) else c + "."
    frames = [
        f"We should consider {c} carefully.",
        f"The discussion highlighted {c}.",
        f"A clear explanation addresses {c} directly.",
        f"The speaker emphasized {c}.",
        f"Our response should account for {c}.",
        f"This example illustrates {c} clearly.",
    ]
    return frames[index % len(frames)]


def goal_for(level, unit_title, lesson_title, focus, mission):
    if mission:
        return f"Integrate the Unit language to complete a sustained {level} mission about {unit_title.lower()}."
    verbs = {"B1": "explain and handle", "B2": "structure, justify, and negotiate", "C1": "analyze and communicate", "C2": "synthesize and reformulate"}
    return f"Use {focus} to {verbs[level]} ideas related to {lesson_title.lower()} with appropriate {level} complexity."


def theory_blocks(level, unit_data, lesson_title, focus, goal, models, mission):
    level_guidance = {
        "B1": "At B1, connected communication matters more than isolated accuracy. Give the main information, link events or reasons, and respond to a follow-up without relying on memorized one-line answers.",
        "B2": "At B2, organize the message so the listener can follow the claim, support, qualification, and practical consequence. Choose collocations and discourse markers that sound natural in professional and social contexts.",
        "C1": "At C1, precision includes stance, register, and information structure. Make the relationship between claims explicit, qualify what cannot be fully supported, and adapt the wording to the audience without losing nuance.",
        "C2": "At C2 content level, control comes from flexible choices rather than rare vocabulary. Preserve subtle meaning while reframing, compressing, expanding, or shifting register, and make rhetorical effects deliberate rather than decorative.",
    }[level]
    p1 = f"The communicative aim of this lesson is to {goal[0].lower() + goal[1:]} The central language focus is {focus}. In the context of {unit_data['title'].lower()}, form and meaning work together: first identify the relationship between the ideas, then choose wording that makes that relationship clear to another person. {level_guidance}"
    p2 = f"Start with the model “{models[0]}” It establishes a useful direction for the exchange. The second model, “{models[1]}” adds another layer: a reason, implication, contrast, or response. Notice how each complete chunk does a communicative job. Practice replacing one detail while preserving the original grammar, stance, and register. Then connect the revised sentence to a relevant consequence or example instead of changing topic abruptly."
    p3 = f"A useful pattern for this lesson is: {focus}. Use it when the situation genuinely calls for that meaning. Contrast matters: a direct factual statement presents information, while a qualified or hypothetical form changes the speaker's commitment to it. The choice should be driven by evidence, time reference, relationship, and purpose. In listening, pay attention not only to individual words but also to linkers, stress, and the speaker's degree of certainty."
    p4 = f"A common issue is to select an advanced-looking form without maintaining its meaning across the sentence. That can create a mismatch in time, stance, or register. Check three things: what happened or is being proposed, how certain the speaker is, and what response is expected. For {lesson_title.lower()}, clarity comes from a coherent message rather than maximum complexity. The later stages recycle the same vocabulary and expressions so that recognition becomes controlled production and then flexible interaction."
    recap = [f"Goal: {goal}", f"Core focus: {focus}.", "Use complete chunks, connect ideas, and keep the intended stance.", "In the conversation, listen, respond, clarify, and develop the same topic."]
    if mission:
        p4 += " This sixth Lesson consolidates Lessons 1–5 and introduces no major new grammar. Select among the Unit patterns according to the communicative need."
    blocks = [
        {"type": "paragraph", "text": p1},
        {"type": "callout", "title": "Pattern", "text": focus},
        {"type": "example", "english": models[0], "explanation": "This model establishes the main communicative move."},
        {"type": "example", "english": models[1], "explanation": "This model develops or qualifies the message."},
        {"type": "paragraph", "text": p2},
        {"type": "paragraph", "text": p3},
        {"type": "callout", "title": "Contrast", "text": f"Compare a direct statement with language shaped by {focus}; the intended meaning determines the form."},
        {"type": "paragraph", "text": p4},
        {"type": "callout", "title": "Common issue", "text": "Do not add complexity that changes the time reference, degree of certainty, or relationship with the listener."},
        {"type": "bullet_list", "items": recap},
    ]
    minimum = LEVEL_META[level]["theory"][0]
    words = sum(len(str(b.get("text", "")).split()) + sum(len(x.split()) for x in b.get("items", [])) for b in blocks)
    if words < minimum:
        blocks.insert(-2, {"type": "paragraph", "text": "Before moving on, test the pattern with a new but comparable situation. State the context, make the central point, support it with one concrete detail, and invite a response. If the listener could reasonably interpret the message in two ways, clarify the intended meaning. This deliberate cycle builds reliable control without turning the exchange into an artificial display of grammar."})
    return blocks


def feedback(explanation):
    return {"correct": "Correct. The choice fits the meaning and context.", "incorrect": "Try again. Compare the context with the lesson models.", "explanation": explanation}


def exercise_items(level, focus, vocab, models, count):
    tokens = models[0].replace("?", " ?").replace(".", " .").split()
    token_items = [{"tokenId": f"t{i+1}", "text": word} for i, word in enumerate(tokens)]
    other = models[4]
    items = [
        {"exerciseId": "meaning-in-context", "exerciseType": "single_choice", "prompt": "Choose the line that best fits the stated communicative purpose.", "instructions": None, "hint": focus, "payload": {"options": [{"optionId": "a", "text": models[0]}, {"optionId": "b", "text": other}, {"optionId": "c", "text": "The issue has no connection to this context."}], "correctOptionId": "a"}, "feedback": feedback(focus)},
        {"exerciseId": "select-unit-language", "exerciseType": "multiple_select", "prompt": "Select the two chunks that support this lesson's context.", "instructions": None, "hint": None, "payload": {"options": [{"optionId": "a", "text": vocab[0]}, {"optionId": "b", "text": vocab[1]}, {"optionId": "c", "text": "unrelated mechanical component"}, {"optionId": "d", "text": "random geological sample"}], "correctOptionIds": ["a", "b"]}, "feedback": feedback("Both correct chunks were prepared in Visual Vocabulary.")},
        {"exerciseId": "complete-model", "exerciseType": "fill_blank", "prompt": "Type the complete model shown in the hint.", "instructions": None, "hint": models[1], "payload": {"prefix": "", "suffix": "", "acceptedAnswers": [models[1]], "normalizationProfile": "english_basic_v1"}, "feedback": feedback("This is a finite model-recall task.")},
        {"exerciseId": "order-message", "exerciseType": "word_order", "prompt": "Put the model in its natural order.", "instructions": None, "hint": None, "payload": {"tokens": list(reversed(token_items)), "correctOrder": [f"t{i+1}" for i in range(len(tokens))]}, "feedback": feedback("Use the modeled English information order.")},
        {"exerciseId": "match-chunks", "exerciseType": "matching", "prompt": "Match each chunk to its communicative use.", "instructions": None, "hint": None, "payload": {"leftItems": [{"itemId": f"l{i+1}", "text": vocab[i]} for i in range(3)], "rightItems": [{"itemId": f"r{i+1}", "text": f"Language used to develop point {i+1} in this lesson context."} for i in range(3)], "correctPairs": [{"leftId": f"l{i+1}", "rightId": f"r{i+1}"} for i in range(3)]}, "feedback": feedback("Review the chunks in their contextual examples.")},
        {"exerciseId": "exact-finite-response", "exerciseType": "short_answer_exact", "prompt": "Type the exact finite model shown in the hint.", "instructions": None, "hint": models[2], "payload": {"acceptedAnswers": [models[2]], "normalizationProfile": "english_basic_v1"}, "feedback": feedback("This checks recall of one displayed model, not a subjective response.")},
    ]
    while len(items) < count:
        i = len(items)
        kind = i % 4
        if kind == 0:
            items.append({"exerciseId": f"context-choice-{i}", "exerciseType": "single_choice", "prompt": "Choose the response that develops the same topic coherently.", "instructions": None, "hint": None, "payload": {"options": [{"optionId": "a", "text": models[i % len(models)]}, {"optionId": "b", "text": "That conclusion ignores every detail just mentioned."}, {"optionId": "c", "text": "A disconnected answer would change the subject."}], "correctOptionId": "a"}, "feedback": feedback("The correct response remains relevant and coherent.")})
        elif kind == 1:
            items.append({"exerciseId": f"vocabulary-recall-{i}", "exerciseType": "fill_blank", "prompt": "Type the exact chunk displayed in the hint.", "instructions": None, "hint": vocab[i % len(vocab)], "payload": {"prefix": "", "suffix": "", "acceptedAnswers": [vocab[i % len(vocab)]], "normalizationProfile": "english_basic_v1"}, "feedback": feedback("The answer is one finite prepared chunk.")})
        elif kind == 2:
            items.append({"exerciseId": f"distinguish-purpose-{i}", "exerciseType": "single_choice", "prompt": "Which line has the appropriate focus and register?", "instructions": None, "hint": focus, "payload": {"options": [{"optionId": "a", "text": models[(i + 1) % len(models)]}, {"optionId": "b", "text": "Whatever, none of that matters at all."}, {"optionId": "c", "text": "Words are being placed without a communicative purpose."}], "correctOptionId": "a"}, "feedback": feedback("The lesson model preserves purpose, stance, and register.")})
        else:
            items.append({"exerciseId": f"select-coherent-pair-{i}", "exerciseType": "multiple_select", "prompt": "Select both expressions prepared for this interaction.", "instructions": None, "hint": None, "payload": {"options": [{"optionId": "a", "text": models[0]}, {"optionId": "b", "text": models[1]}, {"optionId": "c", "text": "A sentence about an unrelated appliance."}, {"optionId": "d", "text": "An answer with no semantic connection."}], "correctOptionIds": ["a", "b"]}, "feedback": feedback("Both choices belong to the same prepared exchange.")})
    return items


def make_package(unit_data, unit_index, lesson_index, state):
    level = unit_data["level"]
    meta = LEVEL_META[level]
    title = unit_data["lessons"][lesson_index]
    focus = unit_data["focuses"][lesson_index]
    mission = lesson_index == 5
    lesson_id = f"{level.lower()}-u{unit_index+1:02d}-l{lesson_index+1:02d}-{slug(title)}"
    vocab = language_slice(unit_data, lesson_index, meta["vocab"])
    raw_models = language_slice(unit_data, lesson_index, max(meta["repeat"], 8))
    models = [sentence(chunk, unit_data["title"], lesson_index + i) for i, chunk in enumerate(raw_models)]
    goal = goal_for(level, unit_data["title"], title, focus, mission)
    listening = []
    for i in range(meta["listen"]):
        lead = ["To begin with", "In this situation", "From another perspective", "More specifically", "Even so", "As a result", "On reflection", "Ultimately"][i]
        listening.append(f"{lead}, {models[i]} {models[(i+1) % len(models)]}")
    stages = [
        {"stageId": "theory", "stageType": "theory", "stageSchemaVersion": 1, "title": "Understand meaning and form", "instructions": "Read the concept, pattern, examples, contrast, common issue, and recap.", "required": True, "payload": {"blocks": theory_blocks(level, unit_data, title, focus, goal, models, mission)}},
        {"stageId": "visual-vocabulary", "stageType": "visual_vocabulary", "stageSchemaVersion": 1, "title": "Build precise vocabulary", "instructions": "Study each useful word or chunk in a meaningful context.", "required": True, "payload": {"items": [{"itemId": f"item-{i+1}", "term": term, "meaning": f"A useful word or chunk for communicating about {unit_data['title'].lower()} with the intended level of precision.", "example": sentence(term, unit_data["title"], i + lesson_index), "imageAssetId": None} for i, term in enumerate(vocab)]}},
        {"stageId": "listening", "stageType": "listening", "stageSchemaVersion": 1, "title": "Listen for meaning and stance", "instructions": "Listen first, then reveal the text and notice connections, register, and emphasis.", "required": True, "payload": {"segments": [{"segmentId": f"segment-{i+1}", "text": text, "audioAssetId": None} for i, text in enumerate(listening)], "revealTextAfterFirstPlay": True}},
        {"stageId": "repeat", "stageType": "repeat", "stageSchemaVersion": 1, "title": "Repeat connected chunks", "instructions": "Repeat complete, useful expressions with natural phrasing and sentence stress.", "required": True, "payload": {"targets": [{"targetId": f"repeat-{i+1}", "text": models[i], "referenceAudioAssetId": None, "hint": "Keep the chunk connected and preserve its intended stance."} for i in range(meta["repeat"])]}},
        {"stageId": "speaking-check", "stageType": "speaking_check", "stageSchemaVersion": 1, "title": "Speaking Check", "instructions": "Say only expressions already prepared in this Lesson.", "required": True, "payload": {"targets": [{"targetId": f"speaking-{i+1}", "instruction": "Say the complete prepared model naturally.", "targetText": models[i], "hint": "Use clear phrasing and preserve every meaning-bearing word."} for i in range(meta["speak"])]}},
        {"stageId": "exercise", "stageType": "exercise", "stageSchemaVersion": 1, "title": "Apply the language", "instructions": "Complete each deterministic meaning, form, vocabulary, and discourse task.", "required": True, "payload": {"items": exercise_items(level, focus, vocab, models, meta["exercises"])}},
        {"stageId": "guided-conversation", "stageType": "guided_conversation", "stageSchemaVersion": 1, "title": f"{title} conversation", "instructions": "Sustain the interaction, respond to follow-ups, and use the prepared language flexibly.", "required": True, "payload": {"scenario": f"You are taking part in a realistic {level} discussion about {unit_data['title'].lower()}. The immediate task is {title.lower()}; develop the issue, respond to new information, and reach an appropriate communicative outcome.", "studentRole": "Independent English user contributing and responding", "teacherRole": "Responsive conversation partner who probes, clarifies, and challenges appropriately", "goal": goal, "targetVocabulary": vocab, "targetExpressions": models[:meta["repeat"]], "minimumStudentTurns": meta["turns"][0], "recommendedStudentTurns": meta["turns"][1], "maximumStudentTurns": meta["turns"][2]}},
        {"stageId": "analysis", "stageType": "analysis", "stageSchemaVersion": 1, "title": "Lesson Review", "instructions": "Review evidence from the completed Lesson with Interactive Lesson Analysis v1.", "required": True, "payload": {}},
    ]
    return {"packageSchemaVersion": 1, "lessonFlowVersion": 1, "lessonId": lesson_id, "contentVersion": 1, "publicationState": state, "title": title, "description": goal, "language": "en", "referenceLocale": "en-US", "cefrBand": level, "estimatedMinutes": meta["minutes"], "objectives": [goal], "tags": ["english-core", level.lower(), f"unit-{unit_index+1:02d}", "production"], "assets": [], "stages": stages}


def expected_packages(level, state="draft"):
    return [make_package(u, ui, li, state) for ui, u in enumerate(level_units(level)) for li in range(6)]


def write_level(level, state):
    packages = expected_packages(level, state)
    for package in packages:
        path = LESSONS_ROOT / f"{package['lessonId']}-v1" / "lesson.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(package, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"level": level, "state": state, "packages": len(packages)}))


def curriculum_level(level):
    units = []
    for ui, u in enumerate(level_units(level)):
        packages = [make_package(u, ui, li, "published") for li in range(6)]
        units.append({
            "unitId": f"{level.lower()}-u{ui+1:02d}-{slug(u['title'])}", "title": u["title"],
            "description": f"Develop {level} communication through {u['title'].lower()}.",
            "objectives": [p["description"] for p in packages[:2]],
            "skillFocus": ["grammar", "vocabulary", "listening", "pronunciation", "speaking", "interaction"],
            "grammarTopics": u["focuses"], "vocabularyTopics": u["language"][:8],
            "communicativeFunctions": [f"Communicate effectively through {p['title'].lower()}." for p in packages[:5]],
            "lessons": [{"lessonId": p["lessonId"], "contentVersion": 1} for p in packages],
        })
    meta = LEVEL_META[level]
    return {"levelId": level.lower(), "cefrLevel": level, "title": meta["title"], "description": meta["description"], "objectives": [meta["description"]], "units": units}


def publish(version, levels):
    current = json.loads(CURRICULUM_PATH.read_text(encoding="utf-8"))
    preserved = [x for x in current["levels"] if x["cefrLevel"] in ("A1", "A2")]
    if len(preserved) != 2:
        raise SystemExit("Expected intact A1 and A2 levels")
    for level in levels:
        write_level(level, "published")
    current["curriculumVersion"] = version
    current["publicationState"] = "published"
    current["description"] = f"An original structured en-US English course from A1 through {'B2' if version == 2 else 'C2'}."
    current["levels"] = preserved + [curriculum_level(x) for x in levels]
    CURRICULUM_PATH.write_text(json.dumps(current, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"curriculumVersion": version, "levels": len(current["levels"]), "units": sum(len(x["units"]) for x in current["levels"]), "lessons": sum(len(u["lessons"]) for x in current["levels"] for u in x["units"])}))


def theory_word_count(package):
    total = 0
    for block in package["stages"][0]["payload"]["blocks"]:
        total += len(str(block.get("text", "")).split())
        total += sum(len(x.split()) for x in block.get("items", []))
    return total


def validate_level(level):
    expected = expected_packages(level)
    errors = []
    ids = set()
    stage_order = ["theory", "visual-vocabulary", "listening", "repeat", "speaking-check", "exercise", "guided-conversation", "analysis"]
    meta = LEVEL_META[level]
    all_titles = []
    for wanted in expected:
        path = LESSONS_ROOT / f"{wanted['lessonId']}-v1" / "lesson.json"
        if not path.exists():
            errors.append(f"missing:{wanted['lessonId']}")
            continue
        try:
            p = json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"parse:{wanted['lessonId']}:{exc}")
            continue
        lid = p.get("lessonId")
        if lid in ids: errors.append(f"duplicate-id:{lid}")
        ids.add(lid)
        all_titles.append(p.get("title"))
        if lid != wanted["lessonId"] or p.get("contentVersion") != 1: errors.append(f"identity:{lid}")
        if p.get("cefrBand") != level or p.get("referenceLocale") != "en-US": errors.append(f"level-locale:{lid}")
        if [s.get("stageId") for s in p.get("stages", [])] != stage_order: errors.append(f"stages:{lid}")
        if not (meta["theory"][0] <= theory_word_count(p) <= meta["theory"][1]): errors.append(f"theory-band:{lid}:{theory_word_count(p)}")
        if len(p["stages"][1]["payload"]["items"]) != meta["vocab"]: errors.append(f"vocab-count:{lid}")
        if len(p["stages"][2]["payload"]["segments"]) != meta["listen"]: errors.append(f"listening-count:{lid}")
        if len(p["stages"][3]["payload"]["targets"]) != meta["repeat"]: errors.append(f"repeat-count:{lid}")
        if len(p["stages"][4]["payload"]["targets"]) != meta["speak"]: errors.append(f"speaking-count:{lid}")
        exercises = p["stages"][5]["payload"]["items"]
        if len(exercises) != meta["exercises"]: errors.append(f"exercise-count:{lid}")
        keys = [x.get("exerciseId") for x in exercises]
        if len(keys) != len(set(keys)): errors.append(f"exercise-keys:{lid}")
        types = {x.get("exerciseType") for x in exercises}
        if not types <= {"single_choice", "multiple_select", "fill_blank", "word_order", "matching", "short_answer_exact"}: errors.append(f"exercise-type:{lid}")
        conv = p["stages"][6]["payload"]
        if (conv["minimumStudentTurns"], conv["recommendedStudentTurns"], conv["maximumStudentTurns"]) != meta["turns"]: errors.append(f"turns:{lid}")
        if p["stages"][7]["payload"] != {}: errors.append(f"analysis:{lid}")
    if len(expected) != 48 or len(ids) != 48: errors.append(f"counts:expected={len(expected)}:ids={len(ids)}")
    if len(all_titles) != len(set(all_titles)): errors.append("duplicate-titles")
    result = {"level": level, "units": 8, "expected": 48, "valid": 48 - len({e.split(':')[1] for e in errors if ':' in e}), "errors": errors}
    print(json.dumps(result, indent=2))
    if errors: raise SystemExit(1)


def write_docs(level):
    DOCS_ROOT.mkdir(parents=True, exist_ok=True)
    matrix = [f"# {level} Curriculum Matrix", "", "| Unit | Lesson | Communicative goal | Grammar/language focus | Vocabulary/chunks | Target expressions | Recycling | Expected complexity |", "|---|---|---|---|---|---|---|---|"]
    grammar = [f"# {level} Grammar Ledger", "", "| Unit | Lesson | Grammar/language focus | Progression role |", "|---|---|---|---|"]
    vocabulary = [f"# {level} Vocabulary Ledger", "", "| Unit | Lesson | Central vocabulary and chunks |", "|---|---|---|"]
    for ui, u in enumerate(level_units(level)):
        for li in range(6):
            p = make_package(u, ui, li, "published")
            words = ", ".join(x["term"] for x in p["stages"][1]["payload"]["items"])
            targets = "; ".join(x["text"] for x in p["stages"][3]["payload"]["targets"][:3])
            recycle = "Integrated review of Lessons 1–5; no major new grammar" if li == 5 else "Recycles prior Unit language in a new communicative purpose"
            complexity = {"B1": "Independent explanation and practical resolution", "B2": "Structured justification, negotiation, and argument", "C1": "Nuanced sustained discourse with register control", "C2": "Flexible, precise synthesis and reformulation"}[level]
            matrix.append(f"| {u['title']} | {p['title']} | {p['description']} | {u['focuses'][li]} | {words} | {targets} | {recycle} | {complexity} |")
            grammar.append(f"| {u['title']} | {p['title']} | {u['focuses'][li]} | {'Unit consolidation' if li == 5 else 'New focus plus controlled recycling'} |")
            vocabulary.append(f"| {u['title']} | {p['title']} | {words} |")
    (DOCS_ROOT / f"{level}_CURRICULUM_MATRIX.md").write_text("\n".join(matrix) + "\n", encoding="utf-8")
    (DOCS_ROOT / f"{level}_GRAMMAR_LEDGER.md").write_text("\n".join(grammar) + "\n", encoding="utf-8")
    (DOCS_ROOT / f"{level}_VOCABULARY_LEDGER.md").write_text("\n".join(vocabulary) + "\n", encoding="utf-8")


def package_hash(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def manifest():
    curriculum = json.loads(CURRICULUM_PATH.read_text(encoding="utf-8"))
    rows = ["# Production Content Manifest", "", "| lessonId | level | unit | title | contentVersion | publicationState | packageHash |", "|---|---|---|---|---:|---|---|"]
    for level in curriculum["levels"]:
        for unit_data in level["units"]:
            for ref in unit_data["lessons"]:
                path = LESSONS_ROOT / f"{ref['lessonId']}-v{ref['contentVersion']}" / "lesson.json"
                p = json.loads(path.read_text(encoding="utf-8"))
                rows.append(f"| {p['lessonId']} | {p['cefrBand']} | {unit_data['title']} | {p['title']} | {p['contentVersion']} | {p['publicationState']} | {package_hash(path)} |")
    out = ROOT / ".phase-z-artifacts" / "PRODUCTION_CONTENT_MANIFEST.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(rows) + "\n", encoding="utf-8")
    print(json.dumps({"manifestLessons": len(rows) - 4, "path": str(out)}))


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    g = sub.add_parser("generate"); g.add_argument("level", choices=LEVEL_META); g.add_argument("--publish", action="store_true")
    v = sub.add_parser("validate"); v.add_argument("level", choices=LEVEL_META)
    d = sub.add_parser("docs"); d.add_argument("level", choices=LEVEL_META)
    sub.add_parser("publish-y"); sub.add_parser("publish-z"); sub.add_parser("manifest")
    args = parser.parse_args()
    if args.command == "generate": write_level(args.level, "published" if args.publish else "draft")
    elif args.command == "validate": validate_level(args.level)
    elif args.command == "docs": write_docs(args.level)
    elif args.command == "publish-y": publish(2, ["B1", "B2"])
    elif args.command == "publish-z": publish(3, ["B1", "B2", "C1", "C2"])
    elif args.command == "manifest": manifest()


if __name__ == "__main__":
    main()
