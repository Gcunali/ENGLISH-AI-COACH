import json
from pathlib import Path

ROOT = Path(r"C:\ENGLISH AI COACH")
LESSON_ROOT = ROOT / "src-tauri" / "resources" / "interactive-lessons"
CURRICULUM_ROOT = ROOT / "src-tauri" / "resources" / "curriculum" / "english-core"

LESSONS = [
    {
        "id": "a1-u01-l01-hello-goodbye",
        "title": "Hello & Goodbye",
        "goal": "Greet someone, respond naturally, and end a short conversation politely.",
        "grammar": "fixed greeting expressions and subject pronouns I and you",
        "vocab": [
            ("hello", "a neutral greeting", "Hello, I'm Maya."),
            ("hi", "a friendly informal greeting", "Hi, Leo!"),
            ("good-morning", "a greeting used in the morning", "Good morning, Mr. Lee."),
            ("good-afternoon", "a greeting used after midday", "Good afternoon, Ana."),
            ("good-evening", "a greeting used in the evening", "Good evening, everyone."),
            ("goodbye", "a neutral expression when leaving", "Goodbye, and have a good day."),
            ("see-you", "a friendly expression before leaving", "See you tomorrow!"),
            ("nice-to-meet-you", "a polite response at a first meeting", "Nice to meet you, Sam."),
        ],
        "expressions": [
            "Hello! How are you?",
            "Hi! I'm fine, thank you.",
            "Nice to meet you.",
            "Goodbye! Have a good day.",
            "See you tomorrow!",
        ],
        "explanation": "Greetings open a conversation and goodbyes close it. Choose the expression for the time and relationship. Hello is neutral, while hi is more informal. Good morning, good afternoon, and good evening refer to the time of day. After a first introduction, Nice to meet you is a polite response. When you leave, goodbye is neutral and See you is friendly.",
        "pattern": "Greeting + short response; goodbye + optional good wish",
        "common": "Do not use good night to greet someone. In en-US, good night normally closes an evening conversation or comes before sleep.",
    },
    {
        "id": "a1-u01-l02-whats-your-name",
        "title": "What's Your Name?",
        "goal": "Ask for a person's name, give your own name, and check pronunciation politely.",
        "grammar": "What is, my, your, and the short form what's",
        "vocab": [
            ("first-name", "the name used before a family name", "My first name is Sofia."),
            ("last-name", "a family name", "My last name is Carter."),
            ("full-name", "first and last name together", "My full name is Sofia Carter."),
            ("name", "the word people use to identify you", "What's your name?"),
            ("spell", "say or write the letters in order", "How do you spell Mia?"),
            ("letter", "one symbol in the alphabet", "The first letter is M."),
            ("call", "use a preferred name for someone", "Please call me Ben."),
            ("again", "one more time", "Can you say that again?"),
        ],
        "expressions": [
            "What's your name?",
            "My name is Jordan.",
            "Please call me Jo.",
            "How do you spell that?",
            "Can you say that again, please?",
        ],
        "explanation": "Use What's your name? in a friendly first meeting. What's is the common short form of What is. Answer with My name is plus your name, or say I'm plus your name. My shows that the name belongs to the speaker; your refers to the other person. If a name is unclear, ask How do you spell that? or Can you say that again, please? These questions are polite and practical.",
        "pattern": "What's your name? — My name is ... / I'm ...",
        "common": "Do not answer My name are. Name is singular, so use My name is. Also keep my and your separate.",
    },
    {
        "id": "a1-u01-l03-countries-nationalities",
        "title": "Countries & Nationalities",
        "goal": "Say where you are from, state your nationality, and ask another person about origin.",
        "grammar": "verb be plus from, and adjective forms for nationality",
        "vocab": [
            ("country", "a nation and its territory", "Brazil is a large country."),
            ("nationality", "the national group a person belongs to", "My nationality is Mexican."),
            ("language", "a system people use to communicate", "English is a language."),
            ("brazil", "a country in South America", "I'm from Brazil."),
            ("brazilian", "from Brazil", "I'm Brazilian."),
            ("mexico", "a country in North America", "She's from Mexico."),
            ("mexican", "from Mexico", "She's Mexican."),
            ("united-states", "a country in North America", "He's from the United States."),
            ("american", "from the United States", "He's American."),
        ],
        "expressions": [
            "Where are you from?",
            "I'm from Brazil.",
            "I'm Brazilian.",
            "What language do you speak?",
            "I speak Portuguese and English.",
        ],
        "explanation": "Use be from plus a country to talk about origin: I'm from Brazil. Use be plus a nationality adjective to describe nationality: I'm Brazilian. A country and a nationality are related, but the words can be different. Ask Where are you from? to learn a person's country. Ask What language do you speak? when you want to know about language. Country, nationality, and language are separate ideas.",
        "pattern": "Subject + be + from + country; Subject + be + nationality adjective",
        "common": "Do not say I am from Brazilian. After from, use the country Brazil. Without from, use the adjective Brazilian.",
    },
    {
        "id": "a1-u01-l04-personal-information",
        "title": "Personal Information",
        "goal": "Exchange basic age, city, phone, and email information in a safe everyday context.",
        "grammar": "basic WH questions with what, where, and how old",
        "vocab": [
            ("age", "the number of years a person has lived", "My age is twenty-four."),
            ("city", "a large town", "My city is Austin."),
            ("address", "information that identifies a home or place", "This is my work address."),
            ("phone-number", "the numbers used to call a phone", "My phone number ends in nine."),
            ("email", "an electronic message address", "My email is on the form."),
            ("form", "a document with spaces for information", "Please complete this form."),
            ("live", "have your home in a place", "I live in Denver."),
            ("contact", "communicate with a person", "You can contact me by email."),
        ],
        "expressions": [
            "How old are you?",
            "I'm twenty-four years old.",
            "Where do you live?",
            "I live in Denver.",
            "What's your email address?",
        ],
        "explanation": "Personal information helps people complete a simple form or stay in contact. Ask How old are you? for age and Where do you live? for a city. Ask What's your phone number? or What's your email address? only when the situation makes the question appropriate. Answer in a full short sentence. Protect private information: practice with invented details when you do not need to share real contact information.",
        "pattern": "WH question + be/do; short subject + verb answer",
        "common": "In English, say I am twenty-four years old, not I have twenty-four years. Use be for age.",
    },
    {
        "id": "a1-u01-l05-i-am-you-are-he-is",
        "title": "I Am, You Are, He Is",
        "goal": "Choose the correct present form of be to identify and describe people.",
        "grammar": "subject pronouns and present forms am, is, and are",
        "vocab": [
            ("i", "the speaker", "I am ready."),
            ("you", "the person or people addressed", "You are welcome."),
            ("he", "one male person", "He is my friend."),
            ("she", "one female person", "She is a student."),
            ("we", "the speaker and another person", "We are neighbors."),
            ("they", "two or more other people", "They are teachers."),
            ("friend", "a person you know and like", "Alex is my friend."),
            ("neighbor", "a person who lives near you", "She is my neighbor."),
        ],
        "expressions": [
            "I am a new student.",
            "You are in my class.",
            "He is my friend.",
            "She is from Canada.",
            "We are neighbors.",
            "They are teachers.",
        ],
        "explanation": "The verb be changes with the subject. Use am only with I. Use is with he, she, it, or one person. Use are with you, we, they, or more than one person. A subject pronoun can replace a person's name after the listener knows who the person is. These short sentences help identify people, give nationality, and describe roles or relationships.",
        "pattern": "I am; he/she/it is; you/we/they are",
        "common": "Do not combine a subject with the wrong form, such as I is or they is. Match the form of be to the subject.",
    },
    {
        "id": "a1-u01-l06-introductions-mission",
        "title": "Introductions Mission",
        "goal": "Complete a first meeting by greeting, exchanging names and origins, and closing politely.",
        "grammar": "integrated review of be, subject pronouns, WH questions, my, and your",
        "vocab": [
            ("introduce", "tell people who someone is", "Let me introduce my friend."),
            ("meet", "see and speak to someone for the first time", "It's good to meet you."),
            ("classmate", "a person in the same class", "Rita is my classmate."),
            ("coworker", "a person who works with you", "This is my coworker, Luis."),
            ("welcome", "a friendly word for a new arrival", "Welcome to the group!"),
            ("from", "showing a place of origin", "We're from Chicago."),
            ("speak", "use a language", "They speak Spanish."),
            ("contact", "information or action used to communicate", "Let's exchange contact information."),
        ],
        "expressions": [
            "Hello, my name is Taylor.",
            "Nice to meet you, Taylor.",
            "Where are you from?",
            "I'm from the United States.",
            "This is my classmate, Noor.",
            "See you in class tomorrow!",
        ],
        "explanation": "A complete introduction has a clear sequence. Start with a greeting. Give your name and ask for the other person's name. Add one or two useful details, such as country, city, language, or role. Listen to the answers and respond politely. If another person joins, use This is my classmate or This is my coworker. Close with Nice to meet you, See you, or Goodbye. This mission reviews the Unit; it adds no new major grammar.",
        "pattern": "greeting → name → question → personal detail → polite response → goodbye",
        "common": "Do not ask every personal question immediately. Choose details that fit the situation and keep private contact information optional.",
    },
]


def theory_blocks(spec):
    bridge = (
        f"The practical goal is to {spec['goal'][0].lower() + spec['goal'][1:]} "
        "Use the pattern in a short exchange, notice the other speaker's words, and answer with one clear idea. "
        "The examples below prepare the same language used in listening, pronunciation practice, exercises, and the final conversation."
    )
    return [
        {"type": "paragraph", "text": spec["explanation"]},
        {"type": "callout", "title": "Pattern", "text": spec["pattern"]},
        *[
            {"type": "example", "english": text, "explanation": f"This complete model prepares the learner to say: {text}"}
            for text in spec["expressions"][:3]
        ],
        {"type": "paragraph", "text": bridge},
        {"type": "callout", "title": "Common error", "text": spec["common"]},
        {
            "type": "bullet_list",
            "items": [
                "Choose the expression that fits the person and situation.",
                f"Remember the core pattern: {spec['pattern']}.",
                "Answer clearly, then invite the other person to continue.",
            ],
        },
    ]


def feedback(explanation):
    return {"correct": "Correct. The form fits this introduction.", "incorrect": "Try again and use the lesson pattern.", "explanation": explanation}


def exercises(spec):
    e = spec["expressions"]
    v = spec["vocab"]
    first_tokens = e[0].replace("?", " ?").replace("!", " !").replace(".", " .").split()
    token_items = [{"tokenId": f"t{i+1}", "text": token} for i, token in enumerate(first_tokens)]
    return [
        {"exerciseId":"choose-expression","exerciseType":"single_choice","prompt":"Choose the expression that best fits the lesson goal.","instructions":None,"hint":"Use the practiced communicative chunk.","payload":{"options":[{"optionId":"a","text":e[0]},{"optionId":"b","text":"I don't know this word."},{"optionId":"c","text":"The window is blue."}],"correctOptionId":"a"},"feedback":feedback(spec["pattern"])},
        {"exerciseId":"select-useful-words","exerciseType":"multiple_select","prompt":"Select the two words that belong to this lesson's core situation.","instructions":None,"hint":None,"payload":{"options":[{"optionId":"a","text":v[0][0].replace('-', ' ')},{"optionId":"b","text":v[1][0].replace('-', ' ')},{"optionId":"c","text":"refrigerator"},{"optionId":"d","text":"mountain engine"}],"correctOptionIds":["a","b"]},"feedback":feedback("Both words were introduced in Visual Vocabulary.")},
        {"exerciseId":"complete-chunk","exerciseType":"fill_blank","prompt":"Complete the practiced expression.","instructions":None,"hint":e[1],"payload":{"prefix":"","suffix":"","acceptedAnswers":[e[1]],"normalizationProfile":"english_basic_v1"},"feedback":feedback("Use the full practiced expression.")},
        {"exerciseId":"order-first-expression","exerciseType":"word_order","prompt":"Put the words in the natural order.","instructions":None,"hint":None,"payload":{"tokens":list(reversed(token_items)),"correctOrder":[f"t{i+1}" for i in range(len(token_items))]},"feedback":feedback("English statements and questions follow the modeled order.")},
        {"exerciseId":"match-meaning","exerciseType":"matching","prompt":"Match each lesson word to its meaning.","instructions":None,"hint":None,"payload":{"leftItems":[{"itemId":f"l{i+1}","text":v[i][0].replace('-', ' ')} for i in range(3)],"rightItems":[{"itemId":f"r{i+1}","text":v[i][1]} for i in range(3)],"correctPairs":[{"leftId":f"l{i+1}","rightId":f"r{i+1}"} for i in range(3)]},"feedback":feedback("Review the term, meaning, and example together.")},
        {"exerciseId":"exact-response","exerciseType":"short_answer_exact","prompt":"Type the exact model response shown in the hint.","instructions":None,"hint":e[2],"payload":{"acceptedAnswers":[e[2]],"normalizationProfile":"english_basic_v1"},"feedback":feedback("This item checks one finite model response, not an open opinion.")},
        {"exerciseId":"choose-pattern","exerciseType":"single_choice","prompt":"Which line follows the lesson pattern?","instructions":None,"hint":spec["grammar"],"payload":{"options":[{"optionId":"a","text":e[3]},{"optionId":"b","text":"Is are my."},{"optionId":"c","text":"You am from?"}],"correctOptionId":"a"},"feedback":feedback(spec["grammar"])},
        {"exerciseId":"complete-vocabulary","exerciseType":"fill_blank","prompt":f"Type the vocabulary item that means: {v[3][1]}","instructions":None,"hint":"The item appears in Visual Vocabulary.","payload":{"prefix":"","suffix":"","acceptedAnswers":[v[3][0].replace('-', ' ')],"normalizationProfile":"english_basic_v1"},"feedback":feedback(v[3][1])},
    ]


def package(spec):
    vocab = spec["vocab"]
    expr = spec["expressions"]
    listening = [
        f"{expr[0]} {expr[1]}",
        f"{expr[2]} {expr[3]}",
        f"{expr[-1]} Thanks for the conversation.",
    ]
    return {
        "packageSchemaVersion": 1,
        "lessonFlowVersion": 1,
        "lessonId": spec["id"],
        "contentVersion": 1,
        "publicationState": "draft",
        "title": spec["title"],
        "description": spec["goal"],
        "language": "en",
        "referenceLocale": "en-US",
        "cefrBand": "A1",
        "estimatedMinutes": 24,
        "objectives": [spec["goal"]],
        "tags": ["english-core", "a1", "unit-01", "production"],
        "assets": [],
        "stages": [
            {"stageId":"theory","stageType":"theory","stageSchemaVersion":1,"title":"Understand the language","instructions":"Read the explanation, examples, common error, and recap.","required":True,"payload":{"blocks":theory_blocks(spec)}},
            {"stageId":"visual-vocabulary","stageType":"visual_vocabulary","stageSchemaVersion":1,"title":"Build your vocabulary","instructions":"Study each term, meaning, and example.","required":True,"payload":{"items":[{"itemId":i,"term":i.replace('-', ' '),"meaning":m,"example":x,"imageAssetId":None} for i,m,x in vocab]}},
            {"stageId":"listening","stageType":"listening","stageSchemaVersion":1,"title":"Listen in context","instructions":"Listen to the short exchange before revealing the text.","required":True,"payload":{"segments":[{"segmentId":f"segment-{i+1}","text":text,"audioAssetId":None} for i,text in enumerate(listening)],"revealTextAfterFirstPlay":True}},
            {"stageId":"repeat","stageType":"repeat","stageSchemaVersion":1,"title":"Repeat useful chunks","instructions":"Repeat each complete expression with clear rhythm.","required":True,"payload":{"targets":[{"targetId":f"repeat-{i+1}","text":text,"referenceAudioAssetId":None,"hint":"Keep the expression together as one useful chunk."} for i,text in enumerate(expr[:5])]}},
            {"stageId":"speaking-check","stageType":"speaking_check","stageSchemaVersion":1,"title":"Speaking Check","instructions":"Say the expressions you have already studied.","required":True,"payload":{"targets":[{"targetId":f"speaking-{i+1}","instruction":"Say the complete model naturally.","targetText":text,"hint":"Use the same words and clear sentence stress."} for i,text in enumerate(expr[:4])]}},
            {"stageId":"exercise","stageType":"exercise","stageSchemaVersion":1,"title":"Check your understanding","instructions":"Complete each deterministic practice item.","required":True,"payload":{"items":exercises(spec)}},
            {"stageId":"guided-conversation","stageType":"guided_conversation","stageSchemaVersion":1,"title":"First-meeting conversation","instructions":"Use the lesson language in a practical first meeting.","required":True,"payload":{"scenario":f"You meet a new person in a community English class. Practice {spec['title'].lower()} in a friendly, appropriate exchange.","studentRole":"English learner meeting someone new","teacherRole":"Friendly new classmate","goal":spec["goal"],"targetVocabulary":[i.replace('-', ' ') for i,_,_ in vocab[:8]],"targetExpressions":expr[:6],"minimumStudentTurns":4,"recommendedStudentTurns":6,"maximumStudentTurns":8}},
            {"stageId":"analysis","stageType":"analysis","stageSchemaVersion":1,"title":"Lesson Review","instructions":"Review the evidence from your completed practice.","required":True,"payload":{}},
        ],
    }


def curriculum():
    return {
        "curriculumSchemaVersion": 1,
        "curriculumId": "english-core",
        "curriculumVersion": 1,
        "publicationState": "draft",
        "title": "English Course",
        "description": "A structured original en-US course for practical beginner English.",
        "targetLanguage": "en",
        "referenceLocale": "en-US",
        "levels": [{
            "levelId": "a1",
            "cefrLevel": "A1",
            "title": "Beginner",
            "description": "Build essential English for concrete everyday communication.",
            "objectives": ["Use simple English in familiar personal and everyday situations."],
            "units": [{
                "unitId": "a1-u01-meeting-people",
                "title": "Meeting People",
                "description": "Greet people, exchange basic information, and complete a first introduction.",
                "objectives": ["Introduce yourself and another person.", "Ask and answer basic personal questions."],
                "skillFocus": ["grammar", "vocabulary", "listening", "pronunciation", "speaking", "interaction"],
                "grammarTopics": ["subject pronouns", "verb be", "basic WH questions", "yes/no questions with be", "basic possessive adjectives"],
                "vocabularyTopics": ["greetings", "names", "countries", "nationalities", "languages", "personal information"],
                "communicativeFunctions": ["greeting", "introducing yourself", "asking for basic information", "closing a conversation"],
                "lessons": [{"lessonId": spec["id"], "contentVersion": 1} for spec in LESSONS],
            }],
        }],
    }


for lesson in LESSONS:
    directory = LESSON_ROOT / f"{lesson['id']}-v1"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "lesson.json").write_text(json.dumps(package(lesson), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

CURRICULUM_ROOT.mkdir(parents=True, exist_ok=True)
(CURRICULUM_ROOT / "curriculum.json").write_text(json.dumps(curriculum(), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print(json.dumps({"lessons": len(LESSONS), "curriculum": str(CURRICULUM_ROOT / 'curriculum.json'), "publicationState": "draft"}))
