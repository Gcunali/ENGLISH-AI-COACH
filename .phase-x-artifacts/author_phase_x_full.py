import argparse
import hashlib
import importlib.util
import json
import re
from pathlib import Path

ROOT = Path(r"C:\ENGLISH AI COACH")
LESSON_ROOT = ROOT / "src-tauri" / "resources" / "interactive-lessons"
CURRICULUM_PATH = ROOT / "src-tauri" / "resources" / "curriculum" / "english-core" / "curriculum.json"
DOC_ROOT = ROOT / "docs" / "content"

pilot_spec = importlib.util.spec_from_file_location("phase_x_pilot", ROOT / ".phase-x-artifacts" / "author_phase_x_pilot.py")
pilot = importlib.util.module_from_spec(pilot_spec)
pilot_spec.loader.exec_module(pilot)


def rows(text):
    result = []
    for raw in text.strip().splitlines():
        title, slug, goal, grammar, first, second = [part.strip() for part in raw.split("|")]
        result.append({"title": title, "slug": slug, "goal": goal, "grammar": grammar, "models": [first, second]})
    return result


def vocabulary(text):
    return [tuple(part.strip() for part in raw.split("=", 1)) for raw in text.strip().splitlines()]


UNITS = [
    {
        "level":"A1","number":2,"title":"Numbers, Time & Dates","slug":"numbers-time-dates",
        "description":"Use numbers, spelling, dates, and clock time to exchange practical details and arrange a meeting.",
        "common":["What time works for you?","That time works for me."],
        "vocab":vocabulary("""
number=a symbol or word used for an amount
zero=the number before one
hundred=the number after ninety-nine
spell=say or write letters in order
phone number=the digits used to call a phone
email address=the address used for electronic messages
weekday=a day from Monday through Friday
weekend=Saturday and Sunday
date=a specific day, month, and year
o'clock=used for an exact clock hour
noon=twelve o'clock in the day
appointment=a planned meeting at a set time
schedule=a plan showing activities and times
available=free and able to meet
"""),
        "lessons":rows("""
Numbers 0–100|numbers-0-100|Say, understand, and use numbers from zero to one hundred in everyday exchanges.|cardinal numbers 0–100|The total is forty-two.|My number is eighty-six.
Spelling, Phones & Emails|spelling-phones-emails|Spell names and exchange a practice phone number and email address clearly.|alphabet spelling and digit-by-digit numbers|How do you spell your last name?|My practice email is maya@example.com.
Days, Months & Dates|days-months-dates|Ask for and state weekdays, months, and simple calendar dates.|What day/date and ordinal date forms|What day is the appointment?|It's on Monday, March fifth.
What Time Is It?|what-time-is-it|Ask for and tell common clock times.|What time, o'clock, half past, and time prepositions|What time is it?|It's half past three.
My Schedule|my-schedule|Describe a simple daily schedule with days and times.|present simple with time expressions|I work at nine on weekdays.|My English class is on Tuesday.
Making a Simple Appointment|making-simple-appointment|Arrange a simple appointment by checking a day and time.|integrated numbers, dates, time, and availability|Are you available on Friday?|Yes, ten o'clock works for me.
""")
    },
    {
        "level":"A1","number":3,"title":"Family & People","slug":"family-people",
        "description":"Identify family members, describe people, and express basic possession.",
        "common":["Who is this?","This is someone in my family."],
        "vocab":vocabulary("""
parent=a mother or father
mother=a female parent
father=a male parent
sister=a female sibling
brother=a male sibling
child=a young son or daughter
grandparent=a parent of a parent
family=people related to one another
have=possess or be connected with
tall=having greater than average height
friendly=kind and pleasant to other people
hair=the strands growing on a person's head
belong=be owned by someone
photo=a picture made with a camera
"""),
        "lessons":rows("""
My Family|my-family|Name close family members and say how they are related.|family nouns and possessive adjectives|This is my sister, Elena.|We are a small family.
Who Is This?|who-is-this|Ask and answer who a person is in a photo or group.|Who is and demonstrative this|Who is this in the photo?|He is my grandfather.
Have & Has|have-has|Say what people have and ask about family or personal features.|have with I/you/we/they and has with he/she/it|I have two brothers.|She has brown eyes.
Describing People|describing-people|Give a short respectful description of a person's appearance and character.|be and have for descriptions|My cousin is friendly and tall.|He has short black hair.
Possession|possession|Say who everyday objects and family items belong to.|possessive adjectives and apostrophe-s|This is Ana's photo.|Their house is near our house.
Meet My Family|meet-my-family|Introduce family members and combine relationship, possession, and description language.|integrated family description review|This is my mother, Grace.|She is friendly and she has curly hair.
""")
    },
    {
        "level":"A1","number":4,"title":"Daily Life","slug":"daily-life",
        "description":"Talk about routines, frequency, and simple present questions and negatives.",
        "common":["What do you do every day?","That's part of my routine."],
        "vocab":vocabulary("""
wake up=stop sleeping
get dressed=put clothes on
breakfast=the first meal of the day
commute=travel regularly to work or school
work=do a job
study=learn about a subject
lunch=a midday meal
exercise=physical activity for health
relax=rest and become less tense
always=every time
usually=most of the time
sometimes=on some occasions
never=at no time
routine=a regular sequence of actions
"""),
        "lessons":rows("""
My Daily Routine|my-daily-routine|Describe the main actions in a typical day in sequence.|present simple first-person routine verbs|I wake up at seven.|I have breakfast before work.
Present Simple|present-simple|Use the present simple for regular actions and facts.|present simple affirmative and third-person -s|I study English every day.|Mina works at a hospital.
I Don't / He Doesn't|dont-doesnt|Make clear present simple negative statements about routines.|do not and does not plus base verb|I don't work on Sunday.|He doesn't drink coffee.
Do You...? / Does She...?|do-you-does-she|Ask and answer present simple questions about regular activities.|do/does questions and short answers|Do you exercise every day?|Yes, I do, but she doesn't.
Always, Usually, Sometimes|frequency-adverbs|Describe how often routine activities happen.|frequency adverbs before main verbs and after be|I usually eat lunch at noon.|We are sometimes busy on Friday.
A Day in My Life|day-in-my-life|Give an integrated short account of a normal day and ask about another person's routine.|integrated present simple and frequency review|I usually start work at nine.|What do you do after dinner?
""")
    },
    {
        "level":"A1","number":5,"title":"Home & Around Town","slug":"home-around-town",
        "description":"Describe a home and neighborhood, locate places, and give short directions.",
        "common":["Where is it?","It's near here."],
        "vocab":vocabulary("""
living room=a room for relaxing and receiving visitors
kitchen=a room where food is prepared
bedroom=a room for sleeping
bathroom=a room with a toilet or bath
furniture=movable items such as tables and chairs
there is=phrase showing one thing exists in a place
there are=phrase showing several things exist in a place
next to=at the side of something
across from=on the opposite side
between=in the middle of two things
bank=a place that manages money
pharmacy=a store that sells medicine
intersection=a place where roads cross
turn=change direction
"""),
        "lessons":rows("""
My Home|my-home|Name rooms and give a simple description of a home.|be, have, and room nouns|My home has two bedrooms.|The kitchen is next to the living room.
There Is / There Are|there-is-there-are|Say what exists in a room or neighborhood.|there is for singular and there are for plural|There is a table by the window.|There are three chairs in the kitchen.
Where Is It?|where-is-it|Ask for and state an object's location using basic prepositions.|where questions and place prepositions|Where is the key?|It's on the table next to the lamp.
Places in Town|places-in-town|Identify useful places and say where they are in town.|place nouns and there is/are review|There is a pharmacy on Main Street.|The bank is across from the park.
Giving Directions|giving-directions|Give and follow short walking directions.|imperatives, turn, go straight, and location phrases|Go straight for one block.|Turn left at the intersection.
Finding a Place|finding-a-place|Ask for a destination and follow an integrated set of simple directions.|integrated town and direction review|Excuse me, where is the pharmacy?|Go straight and it's on your right.
""")
    },
    {
        "level":"A1","number":6,"title":"Food & Café English","slug":"food-cafe-english",
        "description":"Talk about food preferences and quantities and place a polite café order.",
        "common":["What would you like?","I'd like that, please."],
        "vocab":vocabulary("""
water=a clear drink with no color
coffee=a dark drink made from roasted beans
tea=a drink made by adding hot water to leaves
bread=a baked food made from flour
rice=small grains cooked as food
fruit=the edible part of many plants, often sweet
vegetable=a plant or part of a plant eaten as food
menu=a list of food and drinks available
some=an unspecified amount or number
any=an unspecified amount used often in questions and negatives
hungry=wanting food
thirsty=wanting a drink
order=ask a business to provide food or drink
check=a bill showing what to pay
"""),
        "lessons":rows("""
Food & Drinks|food-drinks|Name common foods and drinks and say what you eat or drink.|food nouns and present simple|I drink water with lunch.|We eat rice and vegetables.
Countable & Uncountable|countable-uncountable|Distinguish items that can be counted from food substances and liquids.|a/an, plural nouns, and uncountable nouns|I have an apple and two sandwiches.|We need some rice and water.
Some & Any|some-any|Ask about and describe available food with some and any.|some in affirmatives and any in questions/negatives|There is some bread on the table.|Do we have any fruit?
What Do You Like?|what-do-you-like|Ask about and state food preferences.|like, don't like, and preference questions|What food do you like?|I like tea, but I don't like coffee.
I'd Like...|id-like|Make a polite simple request for food or a drink.|would like as a fixed polite request|I'd like a cup of tea, please.|Could I have some water?
At the Café|at-the-cafe|Order food and a drink, respond to a question, and ask for the check.|integrated café-order review|I'd like a sandwich and coffee, please.|Could I have the check, please?
""")
    },
    {
        "level":"A1","number":7,"title":"Free Time & Abilities","slug":"free-time-abilities",
        "description":"Talk about hobbies, abilities, current activities, invitations, and simple plans.",
        "common":["What do you like to do?","That sounds fun."],
        "vocab":vocabulary("""
hobby=an activity done regularly for enjoyment
read=look at and understand written words
cook=prepare food with heat
dance=move rhythmically to music
swim=move through water using the body
play=take part in a game or use an instrument
can=be able to do something
free time=time with no required work
right now=at the present moment
invite=ask someone to join an activity
movie=a story shown on a screen
park=a public green area
weekend=Saturday and Sunday
plan=an intention about a future action
"""),
        "lessons":rows("""
Hobbies & Free Time|hobbies-free-time|Name hobbies and describe regular free-time activities.|like plus noun or to-infinitive|I like to read after dinner.|We play games on the weekend.
Can You...?|can-you|Ask about and state basic abilities with can.|can/can't and ability questions|Can you swim?|Yes, I can, but I can't dive.
Likes & Dislikes|likes-dislikes|Compare simple likes and dislikes and ask about preferences.|like/love/don't like plus activities|She loves dancing.|They don't like cooking.
What Are You Doing?|what-are-you-doing|Ask and say what is happening at the present moment.|present continuous for actions now|What are you doing right now?|I'm reading a new book.
Let's Do Something|lets-do-something|Make, accept, or decline a simple activity suggestion.|let's and invitation responses|Let's watch a movie tonight.|Sorry, I can't, but Saturday is good.
Weekend Plans|weekend-plans|Discuss abilities, preferences, current actions, and a simple weekend plan.|integrated free-time review|Can you play tennis on Saturday?|Yes, let's meet at the park.
""")
    },
    {
        "level":"A1","number":8,"title":"Shopping & Everyday English","slug":"shopping-everyday-english",
        "description":"Discuss clothes, colors, sizes, prices, choices, help requests, and common problems.",
        "common":["Can you help me, please?","Yes, of course."],
        "vocab":vocabulary("""
shirt=a piece of clothing for the upper body
pants=clothing covering each leg from the waist
dress=a one-piece garment covering the body
shoes=outer coverings worn on the feet
color=the appearance described as red, blue, or another hue
size=a measurement showing how large something is
price=the amount of money something costs
cheap=costing little money
expensive=costing a lot of money
this=the near thing being indicated
that=the more distant thing being indicated
try on=put on clothing to test its fit
receipt=a paper or digital record of a purchase
broken=damaged and not working correctly
"""),
        "lessons":rows("""
Clothes & Colors|clothes-colors|Name basic clothing and describe its color.|be plus color and clothing nouns|This shirt is blue.|Her shoes are black.
Sizes & Prices|sizes-prices|Ask for a clothing size and understand a simple price.|what size/how much and number review|What size do you need?|How much is this jacket?
This One or That One?|this-one-that-one|Compare and choose between nearby and more distant items.|this/that and one as a substitute noun|Do you prefer this one or that one?|I like the red one.
Can You Help Me?|can-you-help-me|Ask a store worker for an item, size, or fitting room politely.|can/could requests in a store|Can you help me find a medium?|Could I try this on?
Everyday Problems|everyday-problems|State a simple problem and ask for practical help.|be negative, doesn't work, and help requests|This phone charger is broken.|The card doesn't work.
A1 Real-Life Mission|a1-real-life-mission|Complete a shopping exchange and solve one simple everyday problem.|integrated A1 shopping and help review|I'd like this shirt in blue, please.|It's too small; can you help me?
""")
    },
    {
        "level":"A2","number":1,"title":"Talking About the Past","slug":"talking-about-past",
        "description":"Describe past states and completed actions and ask what happened.",
        "common":["What happened?","Let me tell you about it."],
        "vocab":vocabulary("""
yesterday=the day before today
last night=the evening before today
weekend=Saturday and Sunday
ago=before the present time by a stated amount
visit=go to see a person or place
arrive=reach a place
leave=go away from a place
buy=get something by paying money
meet=come together with someone
happen=take place
regular verb=a verb usually forming the past with ed
irregular verb=a verb with a special past form
remember=bring a past event back to mind
trip=a journey to another place
"""),
        "lessons":rows("""
Was & Were|was-were|Describe where people were and how situations felt in the past.|was/were in statements, negatives, and questions|I was at home last night.|Were you tired after the trip?
Past Simple: Regular Verbs|past-simple-regular|Describe completed past actions with regular verbs and time markers.|regular past -ed and pronunciation patterns|We visited the museum yesterday.|She arrived at nine o'clock.
Common Irregular Verbs|common-irregular-verbs|Use frequent irregular past forms in a short account.|went, had, saw, made, took, came, and bought|I went downtown and bought a book.|We had lunch with our friends.
Did You...?|did-you|Ask and answer about completed past actions.|did/didn't plus base verb|Did you enjoy the concert?|No, I didn't stay until the end.
Yesterday & Last Weekend|yesterday-last-weekend|Connect several past actions to describe yesterday or a weekend.|past simple sequence with time expressions|Yesterday, I worked and then cooked dinner.|Last weekend, we saw our family.
Tell Me What Happened|tell-what-happened|Give and clarify an integrated account of a simple past event.|integrated past simple review|First, I missed the bus; then I called a friend.|What did you do after that?
""")
    },
    {
        "level":"A2","number":2,"title":"Stories & Experiences","slug":"stories-experiences",
        "description":"Set a scene, connect events, and discuss basic life experiences.",
        "common":["What happened next?","That was an interesting experience."],
        "vocab":vocabulary("""
story=a description of connected events
event=something that happens
scene=the place and situation where events occur
suddenly=quickly and unexpectedly
while=during the time that another action happens
when=at the time that something happens
experience=something a person has done or lived through
ever=at any time in one's life
never=not at any time
already=before now or earlier than expected
detail=a small piece of information
memory=something remembered from the past
adventure=an unusual or exciting experience
continue=begin again or keep going
"""),
        "lessons":rows("""
What Were You Doing?|what-were-you-doing|Describe an action that was in progress at a past moment.|past continuous was/were plus -ing|I was walking home at eight.|What were you doing when I called?
When & While|when-while|Connect a continuing background action with a shorter past event.|past continuous with while and past simple with when|While I was cooking, the phone rang.|I met Ava when I was traveling.
Telling a Story|telling-a-story|Organize a short story with a beginning, sequence, and ending.|past tenses with first, then, suddenly, and finally|First, we arrived at the lake.|Suddenly, it started to rain.
Have You Ever...?|have-you-ever|Ask and answer about life experience without a finished time.|present perfect with ever and never|Have you ever traveled alone?|No, I've never traveled alone.
Life Experiences|life-experiences|Share basic experiences and add a simple past detail.|present perfect experience plus past simple detail|I've visited Canada twice.|I went there with my family last year.
A Memorable Experience|memorable-experience|Tell a connected memorable experience using background, events, and reflection.|integrated past and present perfect review|I was studying abroad when I met my best friend.|It was an experience I'll always remember.
""")
    },
    {
        "level":"A2","number":3,"title":"Plans & the Future","slug":"plans-future",
        "description":"Express intentions, arrangements, decisions, predictions, and possibilities.",
        "common":["What are your plans?","Let's make a clear plan together."],
        "vocab":vocabulary("""
plan=a detailed intention about future action
arrangement=a plan agreed with another person
intention=something a person plans to do
prediction=a statement about what may happen
possibility=something that may happen
book=reserve a place or service
pack=put belongings into a bag for travel
leave=go away from a place
arrive=reach a destination
probably=used when something is likely
maybe=used when something is possible
forecast=a statement about expected weather
decision=a choice made after thinking
destination=the place someone is traveling to
"""),
        "lessons":rows("""
Going To|going-to|State planned intentions based on a decision already made.|be going to plus base verb|I'm going to study tonight.|We're going to visit Boston in June.
Future Arrangements|future-arrangements|Discuss fixed personal arrangements with time and place details.|present continuous for arranged future events|I'm meeting Nina at six tomorrow.|We're flying on Friday morning.
Will|will|Make an immediate decision, offer help, or state a neutral future fact.|will plus base verb and contractions|I'll help you carry that bag.|The meeting will start at ten.
Predictions|predictions|Make and explain simple predictions about the future.|will and going to for predictions|I think prices will increase.|Look at those clouds; it's going to rain.
Maybe, Might & Future Possibilities|maybe-might|Discuss uncertain future possibilities without presenting them as facts.|might/may plus base verb and maybe|We might stay an extra day.|Maybe I'll take the train instead.
Planning a Trip|planning-a-trip|Agree on a trip using intentions, arrangements, decisions, and possibilities.|integrated future forms review|We're going to visit Chicago next month.|We might book a hotel near downtown.
""")
    },
    {
        "level":"A2","number":4,"title":"Comparing & Choosing","slug":"comparing-choosing",
        "description":"Compare people and options, describe extremes, and make a reasoned choice.",
        "common":["Which option do you prefer?","I prefer this one because it fits my needs."],
        "vocab":vocabulary("""
compare=examine how things are similar or different
choice=one option selected from several
option=one possible thing to choose
quality=how good something is
feature=an important part or characteristic
price=the amount something costs
fast=moving or happening quickly
convenient=easy and suitable for a situation
reliable=likely to work well consistently
enough=as much as needed
too=more than is wanted or suitable
similar=almost the same
different=not the same
recommend=suggest something as a good choice
"""),
        "lessons":rows("""
Bigger, Better, Faster|bigger-better-faster|Compare two people, places, or products using common comparative forms.|comparatives with -er, more, and irregular better|This room is bigger than that one.|The train is more comfortable than the bus.
The Best Choice|the-best-choice|Identify an extreme or top option in a group.|superlatives with the -est, the most, and the best|This is the cheapest option.|That hotel has the best location.
As...As|as-as|Describe equal and unequal qualities in two options.|as adjective as and not as adjective as|The small model is as fast as the large one.|The bus isn't as convenient as the train.
Too Much / Not Enough|too-much-not-enough|Explain why an amount or quality does not fit a need.|too much/many and not enough|This bag costs too much.|There aren't enough seats for everyone.
Comparing Options|comparing-options|Compare several features and give reasons for a preference.|comparative linkers and because|Option A is cheaper, but Option B is more reliable.|I prefer B because the battery lasts longer.
Choosing the Best Option|choosing-best-option|Reach an integrated choice by comparing needs, features, limits, and value.|integrated comparison review|This apartment is the most convenient for us.|It's expensive, but it has enough space.
""")
    },
    {
        "level":"A2","number":5,"title":"Travel & Accommodation","slug":"travel-accommodation",
        "description":"Navigate travel services, request information, and solve common airport and hotel problems.",
        "common":["Could you help me with this problem?","Let me check the details for you."],
        "vocab":vocabulary("""
boarding pass=a document allowing a passenger onto a flight
gate=the airport area where passengers board
departure=the act or time of leaving
arrival=the act or time of reaching a place
luggage=bags carried while traveling
reservation=an arrangement to keep a room or seat
check in=register on arrival for a flight or hotel
available=ready to be used or booked
delay=a period of waiting because something is late
canceled=stopped and not happening
receipt=a record showing a payment
single room=a hotel room for one person
already=before now or earlier than expected
yet=up to now, often in questions and negatives
"""),
        "lessons":rows("""
At the Airport|at-the-airport|Find a gate, understand basic departure information, and answer check-in questions.|travel questions and airport instructions|Which gate does the flight leave from?|Boarding begins at gate twelve.
Checking In|checking-in|Check in at a hotel and confirm a reservation and room details.|polite requests and reservation details|I have a reservation under Rivera.|Is breakfast included with the room?
Asking for Travel Information|asking-travel-information|Request and clarify times, platforms, locations, and prices.|indirect polite questions with could and can|Could you tell me when the train leaves?|Do you know which platform I need?
Travel Problems|travel-problems|Report a delay, missing item, or reservation problem and request action.|present/past facts with problem and request language|My flight has been canceled.|My luggage didn't arrive with the flight.
Just, Already & Yet|just-already-yet|Give concise updates about recently completed travel actions.|present perfect with just, already, and yet|I've just checked in.|Has the bus arrived yet?
Solving a Travel Problem|solving-travel-problem|Explain a travel problem, clarify details, and agree on a practical solution.|integrated travel and present perfect review|My room isn't ready yet.|Could you store my luggage until two?
""")
    },
    {
        "level":"A2","number":6,"title":"Work, Study & Responsibilities","slug":"work-study-responsibilities",
        "description":"Discuss work and study duties, rules, advice, and personal goals.",
        "common":["What do you need to do?","I'll organize my responsibilities first."],
        "vocab":vocabulary("""
job=regular paid work
course=a series of lessons in a subject
assignment=a task given as part of study or work
deadline=the latest time a task must be finished
meeting=an organized discussion with people
responsibility=a duty a person is expected to handle
rule=an instruction stating what is allowed
required=necessary according to a rule or need
advice=a suggestion about what someone should do
goal=something a person works to achieve
skill=an ability developed through practice
apply=request formally for a job or program
improve=become or make something better
purpose=the reason for doing something
"""),
        "lessons":rows("""
Jobs & Studies|jobs-studies|Describe current work or study and the main tasks involved.|present simple and work/study vocabulary|I work in customer service.|I'm taking an evening design course.
Have To|have-to|Explain external duties and things that are not necessary.|have to/don't have to for obligation|I have to finish this report today.|We don't have to work on Saturday.
Must & Mustn't|must-mustnt|State strong rules and prohibitions clearly.|must and mustn't for rules|Visitors must sign in at reception.|You mustn't share your password.
Giving Advice|giving-advice|Ask for and give practical work or study advice.|should/shouldn't plus base verb|You should make a weekly study plan.|You shouldn't wait until the deadline.
Goals & Purpose|goals-purpose|Explain a personal goal and the purpose behind an action.|to-infinitive for purpose and want to|I'm taking this course to improve my writing.|I want to apply for a new role.
Work or Study Conversation|work-study-conversation|Discuss responsibilities, rules, advice, and goals in an integrated exchange.|integrated obligation, advice, and purpose review|I have to complete an assignment by Friday.|You should ask your instructor for feedback.
""")
    },
    {
        "level":"A2","number":7,"title":"Health & Everyday Problems","slug":"health-everyday-problems",
        "description":"Describe common symptoms, request pharmacy help, give advice, and discuss likely results.",
        "common":["What's wrong?","I can explain how I feel."],
        "vocab":vocabulary("""
head=the upper part of the body containing the face and brain
throat=the passage inside the neck
stomach=the organ that receives food
back=the rear part of the body from shoulders to hips
fever=a body temperature higher than normal
cough=force air noisily from the lungs
sore=painful or uncomfortable
medicine=a substance used to treat illness
pharmacy=a store that prepares and sells medicine
dose=the measured amount of medicine taken at one time
rest=stop activity to recover energy
symptom=a physical sign of illness
appointment=an arranged time to see a professional
emergency=a serious situation needing immediate action
"""),
        "lessons":rows("""
Parts of the Body|parts-body|Name major body parts and locate simple pain or discomfort.|body nouns and possessive forms|My back hurts after long trips.|I have pain in my left shoulder.
What's Wrong?|whats-wrong-health|Describe common symptoms and answer basic follow-up questions.|have plus symptom and feel plus adjective|I have a sore throat and a cough.|I've felt tired since yesterday.
At the Pharmacy|at-the-pharmacy|Ask a pharmacist for a suitable non-emergency product and understand basic directions.|polite requests and quantity/frequency language|Do you have something for a cough?|How often should I take this medicine?
You Should...|you-should|Give and respond to sensible everyday health advice.|should/shouldn't for advice|You should rest and drink water.|You shouldn't exercise with a high fever.
If You Feel Sick...|if-you-feel-sick|Describe a likely result of a present health condition.|first conditional if plus present, will plus base|If you feel worse, you should call a doctor.|If the fever continues, I'll make an appointment.
Getting Help|getting-help|Explain symptoms, ask for help, and agree on an appropriate next action.|integrated symptom, advice, and conditional review|I've had a headache since this morning.|If it gets worse, I'll visit urgent care.
""")
    },
    {
        "level":"A2","number":8,"title":"Opinions, Technology & Social Life","slug":"opinions-technology-social-life",
        "description":"Express opinions, respond respectfully, describe things with relative clauses, and arrange social plans.",
        "common":["What do you think?","I understand your point."],
        "vocab":vocabulary("""
opinion=a personal belief or judgment
agree=have the same opinion
disagree=have a different opinion
reason=a cause or explanation
device=a piece of electronic equipment
app=a program used on a phone or computer
screen=the surface where digital information appears
message=a piece of information sent to someone
privacy=control over personal information
useful=helpful for a purpose
social=related to meeting and communicating with people
invite=ask someone to attend or join
event=an organized activity or occasion
available=free and able to participate
"""),
        "lessons":rows("""
Giving Your Opinion|giving-opinion|State a clear opinion and support it with a simple reason.|I think, in my opinion, and because|I think this app is useful because it's simple.|In my opinion, phones should stay silent in class.
Agreeing & Disagreeing|agreeing-disagreeing|Respond to an opinion with respectful agreement or disagreement.|agreement phrases and polite contrast|I agree with you about online safety.|I see your point, but I prefer meeting in person.
People, Things & Places|people-things-places|Add identifying information about a person, thing, or place.|basic relative clauses with who, that, and where|A coworker is someone who works with you.|This is the café where we first met.
Technology in Daily Life|technology-daily-life|Describe how a device or app helps or creates an everyday problem.|relative clauses and present tense integration|I use an app that tracks my appointments.|My phone is the device that I use most.
Social Plans|social-plans|Invite someone, negotiate details, and confirm a social arrangement.|future arrangements and invitation language|Are you available for dinner on Friday?|I'm meeting some friends at seven.
A2 Real-Life Mission|a2-real-life-mission|Discuss an opinion, solve a technology detail, and agree on a social plan.|integrated A2 tense and function review|I think the event will be fun, but the tickets are expensive.|Let's meet at the place where we had coffee.
""")
    },
]


def slug(value):
    value = value.lower().replace("&", "and")
    value = re.sub(r"[^a-z0-9]+", "-", value).strip("-")
    return value[:90].rstrip("-")


def rotated_vocab(unit, lesson_index):
    values = unit["vocab"]
    size = 8 if unit["level"] == "A1" else 9
    start = (lesson_index * 2) % len(values)
    return [values[(start + i) % len(values)] for i in range(size)]


def lesson_expressions(unit, lesson):
    support = unit["common"]
    close = f"Thanks for talking about {unit['title'].lower()} with me."
    return lesson["models"] + support + [close]


def theory(unit, lesson, expressions):
    level = unit["level"]
    complexity = (
        "At A1, keep the message short and concrete. One clear sentence is enough before the other person responds."
        if level == "A1" else
        "At A2, connect the main message to a time, reason, contrast, or result. Listen for context before choosing the tense or functional phrase."
    )
    explanation = (
        f"This lesson prepares you to {lesson['goal'][0].lower() + lesson['goal'][1:]} "
        f"The central language is {lesson['grammar']}. It belongs to the practical context of {unit['title'].lower()}. "
        "Focus first on meaning: decide what information the listener needs. Then choose the subject, verb form, and useful detail. "
        f"{complexity} The same language returns in the listening, repeat, speaking, exercises, and guided conversation stages."
    )
    bridge = (
        f"Notice the two models: “{expressions[0]}” and “{expressions[1]}” "
        "They show the pattern in complete communication, not as isolated grammar. Say the model aloud, replace one detail, and check that the new sentence keeps the intended meaning. "
        "When the other person asks a follow-up question, answer directly and add a relevant detail. This makes the exchange natural while keeping the language controlled."
    )
    if level == "A2":
        bridge += " Compare the time and viewpoint in each model, and use a linker when two ideas need a clear relationship. Avoid adding advanced forms when the lesson pattern already communicates the message accurately."
    return [
        {"type":"paragraph","text":explanation},
        {"type":"callout","title":"Pattern","text":lesson["grammar"]},
        {"type":"example","english":expressions[0],"explanation":f"Use this model to begin the {lesson['title'].lower()} exchange."},
        {"type":"example","english":expressions[1],"explanation":f"This {lesson['title'].lower()} model adds a different detail or response."},
        {"type":"example","english":expressions[2],"explanation":f"Use this question or response to keep the {lesson['title'].lower()} exchange moving."},
        {"type":"paragraph","text":bridge},
        {"type":"callout","title":"Common error","text":f"Do not mix the word order or verb form with another pattern. Keep the controlled form for this lesson: {lesson['grammar']}."},
        {"type":"bullet_list","items":[f"Communicative goal: {lesson['goal']}",f"Core pattern: {lesson['grammar']}.","Use one clear model, add a relevant detail, and respond to the other speaker."]},
    ]


def feedback(explanation):
    return {"correct":"Correct. The answer fits the lesson context.","incorrect":"Try again and use the model from this lesson.","explanation":explanation}


def exercise_items(unit, lesson, vocab_items, expressions):
    tokens = expressions[0].replace("?", " ?").replace("!", " !").replace(".", " .").split()
    token_values = [{"tokenId":f"t{i+1}","text":word} for i,word in enumerate(tokens)]
    items = [
        {"exerciseId":"choose-model","exerciseType":"single_choice","prompt":"Choose the model that fits the communicative goal.","instructions":None,"hint":lesson["goal"],"payload":{"options":[{"optionId":"a","text":expressions[0]},{"optionId":"b","text":"The purple engine sleeps quickly."},{"optionId":"c","text":"No information is possible."}],"correctOptionId":"a"},"feedback":feedback(lesson["grammar"])},
        {"exerciseId":"select-core-vocabulary","exerciseType":"multiple_select","prompt":"Select both words that belong to this lesson context.","instructions":None,"hint":unit["title"],"payload":{"options":[{"optionId":"a","text":vocab_items[0][0]},{"optionId":"b","text":vocab_items[1][0]},{"optionId":"c","text":"industrial turbine"},{"optionId":"d","text":"quantum geology"}],"correctOptionIds":["a","b"]},"feedback":feedback("Both correct words appear in Visual Vocabulary.")},
        {"exerciseId":"complete-response","exerciseType":"fill_blank","prompt":"Type the exact practiced response shown in the hint.","instructions":None,"hint":expressions[1],"payload":{"prefix":"","suffix":"","acceptedAnswers":[expressions[1]],"normalizationProfile":"english_basic_v1"},"feedback":feedback("Use the complete model response.")},
        {"exerciseId":"order-model","exerciseType":"word_order","prompt":"Put the model in its natural order.","instructions":None,"hint":None,"payload":{"tokens":list(reversed(token_values)),"correctOrder":[f"t{i+1}" for i in range(len(tokens))]},"feedback":feedback("Keep the modeled English word order.")},
        {"exerciseId":"match-vocabulary","exerciseType":"matching","prompt":"Match each term to its meaning.","instructions":None,"hint":None,"payload":{"leftItems":[{"itemId":f"l{i+1}","text":vocab_items[i][0]} for i in range(3)],"rightItems":[{"itemId":f"r{i+1}","text":vocab_items[i][1]} for i in range(3)],"correctPairs":[{"leftId":f"l{i+1}","rightId":f"r{i+1}"} for i in range(3)]},"feedback":feedback("Review each term with its meaning and example.")},
        {"exerciseId":"exact-useful-chunk","exerciseType":"short_answer_exact","prompt":"Type the finite model chunk shown in the hint.","instructions":None,"hint":expressions[2],"payload":{"acceptedAnswers":[expressions[2]],"normalizationProfile":"english_basic_v1"},"feedback":feedback("This checks one exact practiced chunk, not an open response.")},
        {"exerciseId":"choose-correct-form","exerciseType":"single_choice","prompt":"Which sentence follows the lesson pattern?","instructions":None,"hint":lesson["grammar"],"payload":{"options":[{"optionId":"a","text":expressions[1]},{"optionId":"b","text":"She are did going."},{"optionId":"c","text":"I is to yesterday."}],"correctOptionId":"a"},"feedback":feedback(lesson["grammar"])},
        {"exerciseId":"recall-vocabulary","exerciseType":"fill_blank","prompt":f"Type the term that means: {vocab_items[3][1]}","instructions":None,"hint":"It appears in Visual Vocabulary.","payload":{"prefix":"","suffix":"","acceptedAnswers":[vocab_items[3][0]],"normalizationProfile":"english_basic_v1"},"feedback":feedback(vocab_items[3][1])},
    ]
    if unit["level"] == "A2":
        items.append({"exerciseId":"context-choice","exerciseType":"single_choice","prompt":"Choose the response that adds a relevant detail.","instructions":None,"hint":"A2 responses connect ideas and context.","payload":{"options":[{"optionId":"a","text":expressions[3]},{"optionId":"b","text":"Banana seven because blue."},{"optionId":"c","text":"I have no sentence."}],"correctOptionId":"a"},"feedback":feedback("Use a relevant follow-up in the same situation.")})
    return items


def make_package(unit, lesson, lesson_index, state):
    level = unit["level"]
    lesson_id = f"{level.lower()}-u{unit['number']:02d}-l{lesson_index+1:02d}-{lesson['slug']}"
    vocab_items = rotated_vocab(unit, lesson_index)
    expressions = lesson_expressions(unit, lesson)
    listening = [f"{expressions[0]} {expressions[1]}",f"{expressions[2]} {expressions[3]}",f"{expressions[1]} {expressions[2]}"]
    if level == "A2":
        listening.append(f"{expressions[3]} {expressions[4]}")
    return {
        "packageSchemaVersion":1,"lessonFlowVersion":1,"lessonId":lesson_id,"contentVersion":1,"publicationState":state,
        "title":lesson["title"],"description":lesson["goal"],"language":"en","referenceLocale":"en-US","cefrBand":level,
        "estimatedMinutes":24 if level == "A1" else 30,"objectives":[lesson["goal"]],"tags":["english-core",level.lower(),f"unit-{unit['number']:02d}","production"],"assets":[],
        "stages":[
            {"stageId":"theory","stageType":"theory","stageSchemaVersion":1,"title":"Understand the language","instructions":"Read the explanation, examples, common error, and recap.","required":True,"payload":{"blocks":theory(unit,lesson,expressions)}},
            {"stageId":"visual-vocabulary","stageType":"visual_vocabulary","stageSchemaVersion":1,"title":"Build your vocabulary","instructions":"Study each term, meaning, and contextual example.","required":True,"payload":{"items":[{"itemId":slug(term),"term":term,"meaning":meaning,"example":f"Use “{term}” while you practice {lesson['title'].lower()} in context.","imageAssetId":None} for term,meaning in vocab_items]}},
            {"stageId":"listening","stageType":"listening","stageSchemaVersion":1,"title":"Listen in context","instructions":"Listen before revealing the text, then notice the target pattern.","required":True,"payload":{"segments":[{"segmentId":f"segment-{i+1}","text":text,"audioAssetId":None} for i,text in enumerate(listening)],"revealTextAfterFirstPlay":True}},
            {"stageId":"repeat","stageType":"repeat","stageSchemaVersion":1,"title":"Repeat useful chunks","instructions":"Repeat each complete expression with clear rhythm.","required":True,"payload":{"targets":[{"targetId":f"repeat-{i+1}","text":text,"referenceAudioAssetId":None,"hint":"Keep the words together as one communicative chunk."} for i,text in enumerate(expressions[:5] if level=="A2" else expressions[:5])]}},
            {"stageId":"speaking-check","stageType":"speaking_check","stageSchemaVersion":1,"title":"Speaking Check","instructions":"Say expressions already presented in this Lesson.","required":True,"payload":{"targets":[{"targetId":f"speaking-{i+1}","instruction":"Say the complete model naturally.","targetText":text,"hint":"Use the same words and clear sentence stress."} for i,text in enumerate(expressions[:4])]}},
            {"stageId":"exercise","stageType":"exercise","stageSchemaVersion":1,"title":"Check your understanding","instructions":"Complete each deterministic practice item.","required":True,"payload":{"items":exercise_items(unit,lesson,vocab_items,expressions)}},
            {"stageId":"guided-conversation","stageType":"guided_conversation","stageSchemaVersion":1,"title":f"{unit['title']} conversation","instructions":"Use the prepared language in a practical exchange.","required":True,"payload":{"scenario":f"You are in a realistic everyday situation related to {unit['title'].lower()}. Practice {lesson['title'].lower()} and respond to relevant follow-up questions.","studentRole":"English learner handling the situation","teacherRole":"Helpful conversation partner","goal":lesson["goal"],"targetVocabulary":[term for term,_ in vocab_items],"targetExpressions":expressions,"minimumStudentTurns":4 if level=="A1" else 5,"recommendedStudentTurns":6 if level=="A1" else 7,"maximumStudentTurns":8 if level=="A1" else 10}},
            {"stageId":"analysis","stageType":"analysis","stageSchemaVersion":1,"title":"Lesson Review","instructions":"Review the evidence from your completed practice.","required":True,"payload":{}},
        ]
    }


def pilot_unit(state):
    packages=[]
    for value in pilot.LESSONS:
        p=pilot.package(value)
        p["publicationState"]=state
        packages.append(p)
    unit=pilot.curriculum()["levels"][0]["units"][0]
    return unit,packages


def build(scope,state):
    packages=[]
    levels=[]
    punit,ppackages=pilot_unit(state)
    packages.extend(ppackages)
    selected=[u for u in UNITS if scope=="all" or u["level"]=="A1"]
    for unit in selected:
        for index,lesson in enumerate(unit["lessons"]):
            packages.append(make_package(unit,lesson,index,state))
    for level in (["A1"] if scope=="a1" else ["A1","A2"]):
        level_units=[]
        if level=="A1":
            level_units.append(punit)
        for unit in [u for u in selected if u["level"]==level]:
            refs=[]
            for index,lesson in enumerate(unit["lessons"]):
                refs.append({"lessonId":f"{level.lower()}-u{unit['number']:02d}-l{index+1:02d}-{lesson['slug']}","contentVersion":1})
            level_units.append({"unitId":f"{level.lower()}-u{unit['number']:02d}-{unit['slug']}","title":unit["title"],"description":unit["description"],"objectives":[item["goal"] for item in unit["lessons"][:2]],"skillFocus":["grammar","vocabulary","listening","pronunciation","speaking","interaction"],"grammarTopics":[item["grammar"] for item in unit["lessons"]],"vocabularyTopics":[term for term,_ in unit["vocab"][:6]],"communicativeFunctions":[item["goal"] for item in unit["lessons"][:4]],"lessons":refs})
        levels.append({"levelId":level.lower(),"cefrLevel":level,"title":"Beginner" if level=="A1" else "Elementary","description":"Build essential English for concrete everyday communication." if level=="A1" else "Extend basic English to connected everyday communication and problem solving.","objectives":["Communicate in familiar everyday situations with controlled language." if level=="A1" else "Connect ideas and handle common everyday situations with increasing independence."],"units":level_units})
    curriculum={"curriculumSchemaVersion":1,"curriculumId":"english-core","curriculumVersion":1,"publicationState":state,"title":"English Course","description":"An original structured en-US course for practical A1 and A2 communication.","targetLanguage":"en","referenceLocale":"en-US","levels":levels}
    return packages,curriculum


def write_docs(packages,curriculum):
    DOC_ROOT.mkdir(parents=True,exist_ok=True)
    by_id={p["lessonId"]:p for p in packages}
    for level in curriculum["levels"]:
        code=level["cefrLevel"]
        matrix=[f"# {code} Curriculum Matrix","","| Unit | Lesson | Communicative goal | Grammar | Vocabulary | Target expressions | Recycling |","|---|---|---|---|---|---|---|"]
        grammar=[f"# {code} Grammar Ledger","","| Unit | Lesson | Grammar focus |","|---|---|---|"]
        vocab=[f"# {code} Vocabulary Ledger","","| Unit | Lesson | Central vocabulary |","|---|---|---|"]
        for unit in level["units"]:
            for ref in unit["lessons"]:
                p=by_id[ref["lessonId"]]
                theory_stage=p["stages"][0]
                pattern=next(b["text"] for b in theory_stage["payload"]["blocks"] if b.get("title")=="Pattern")
                words=", ".join(i["term"] for i in p["stages"][1]["payload"]["items"])
                expressions="; ".join(i["text"] for i in p["stages"][3]["payload"]["targets"][:3])
                recycling="Mission review" if "mission" in p["lessonId"] or ref==unit["lessons"][-1] else "Reuses Unit vocabulary and prior beginner functions"
                matrix.append(f"| {unit['title']} | {p['title']} | {p['description']} | {pattern} | {words} | {expressions} | {recycling} |")
                grammar.append(f"| {unit['title']} | {p['title']} | {pattern} |")
                vocab.append(f"| {unit['title']} | {p['title']} | {words} |")
        (DOC_ROOT/f"{code}_CURRICULUM_MATRIX.md").write_text("\n".join(matrix)+"\n",encoding="utf-8")
        (DOC_ROOT/f"{code}_GRAMMAR_LEDGER.md").write_text("\n".join(grammar)+"\n",encoding="utf-8")
        (DOC_ROOT/f"{code}_VOCABULARY_LEDGER.md").write_text("\n".join(vocab)+"\n",encoding="utf-8")


parser=argparse.ArgumentParser()
parser.add_argument("scope",choices=["a1","all"])
parser.add_argument("--publish",action="store_true")
args=parser.parse_args()
state="published" if args.publish else "draft"
packages,curriculum=build(args.scope,state)
for package in packages:
    directory=LESSON_ROOT/f"{package['lessonId']}-v1"
    directory.mkdir(parents=True,exist_ok=True)
    (directory/"lesson.json").write_text(json.dumps(package,indent=2,ensure_ascii=False)+"\n",encoding="utf-8")
CURRICULUM_PATH.parent.mkdir(parents=True,exist_ok=True)
CURRICULUM_PATH.write_text(json.dumps(curriculum,indent=2,ensure_ascii=False)+"\n",encoding="utf-8")
write_docs(packages,curriculum)
print(json.dumps({"scope":args.scope,"state":state,"levels":len(curriculum["levels"]),"units":sum(len(x["units"]) for x in curriculum["levels"]),"lessons":len(packages)}))
