import fs from "node:fs";

const path = new URL("../src-tauri/resources/toeic/item-bank-v1/part2.json", import.meta.url);
const bank = JSON.parse(fs.readFileSync(path, "utf8"));
if (bank.forms.some((form) => form.formId === "toeic-part2-form-b")) {
  throw new Error("Part 2 Form B already exists; refusing to duplicate it.");
}

const authored = [
  ["easy","where","Where should the catering boxes be stored?",["In the pantry beside the kitchen.","Before the guests arrive.","The vegetarian meals are labeled."],"A","A location in the pantry directly answers where the boxes should be stored."],
  ["easy","when","When does the neighborhood shuttle make its last trip?",["It stops across from the library.","At ten thirty this evening.","The driver knows the route."],"B","A specific evening time directly answers when the final trip occurs."],
  ["easy","who","Who is leading the orientation for new volunteers?",["In the community hall.","It begins with a short video.","Marcus from visitor services."],"C","Marcus is the only response that identifies the person leading the orientation."],
  ["easy","yes_no","Is the east elevator operating again?",["Yes, the technician fixed it this morning.","The offices are on the east side.","Take the stairs to the lobby."],"A","The response confirms that the elevator is operating and adds when it was repaired."],
  ["easy","request","Could you add this receipt to my expense claim?",["The claim deadline is Friday.","Sure, leave it on my desk.","It came from the airport cafe."],"B","The speaker accepts the request and explains where to leave the receipt."],
  ["easy","what","What comes with the conference registration fee?",["Registration closes next Monday.","At the convention center.","Lunch and a printed workbook."],"C","Lunch and a workbook identify what is included in the registration fee."],
  ["easy","choice","Would you like the receipt by text message or by email?",["Email would be better, thanks.","Yes, I kept the receipt.","The message arrived yesterday."],"A","The response selects email from the two delivery options offered."],
  ["medium","why","Why is the loading entrance closed this afternoon?",["The delivery van is blue.","A safety inspection is taking place there.","Use the entrance near the fountain."],"B","The safety inspection supplies the reason the loading entrance is closed."],
  ["medium","offer","Shall I reserve a projector for your workshop?",["The workshop was well attended.","It projects onto that wall.","Please do; the room does not have one."],"C","The response accepts the offer and explains why a projector is needed."],
  ["medium","how","How did the design team share the revised drawings?",["They uploaded them to the project folder.","The revision took all afternoon.","In the west design studio."],"A","Uploading the drawings to a shared folder explains the method used."],
  ["medium","suggestion","Why don't we ask the supplier for a sample first?",["The supplier moved to a larger warehouse.","Good idea; then we can check the color.","The first shipment arrived by rail."],"B","Checking the color is a natural benefit of accepting the suggestion to request a sample."],
  ["medium","statement","The lobby display is still showing last month's schedule.",["The schedule has five sections.","Last month was unusually busy.","I'll send the updated file to reception."],"C","Sending the updated file is a practical response to the outdated lobby display."],
  ["medium","which","Which applicant should receive the technical exercise?",["The two finalists for the analyst role.","By the end of the business day.","The exercise has three sections."],"A","The response identifies which applicants should receive the exercise."],
  ["medium","negative","Didn't Rosa confirm the table reservation?",["The restaurant has outdoor tables.","No, she's waiting for the final guest count.","We ate there during the conference."],"B","The response says confirmation has not happened and gives the reason for the delay."],
  ["medium","indirect","Do you happen to know where the spare access cards are?",["Security issued my card in June.","They open the laboratory doors.","Try the locked drawer beneath the printer."],"C","The suggested drawer provides the requested location indirectly and naturally."],
  ["medium","request","Please remind the interns to sign the attendance sheet.",["Of course, I'll tell them before the session.","The sheet has the company logo.","They attended last week's seminar."],"A","The response agrees to relay the reminder before the relevant session."],
  ["medium","what","What caused the online payment to be rejected?",["The payment page has a new design.","The billing address did not match the card.","I paid the balance on Tuesday."],"B","A mismatched billing address directly explains why the payment was rejected."],
  ["medium","confirmation","The replacement filters were ordered yesterday, weren't they?",["The filters remove fine dust.","Yesterday's delivery arrived early.","Yes, and the supplier has already shipped them."],"C","The response confirms the order and supplies its current shipping status."],
  ["medium","how","How often are the emergency lights tested?",["Once every three months.","By the building supervisor.","Near each emergency exit."],"A","Once every three months gives the frequency requested by how often."],
  ["hard","indirect","I wonder whether the board has reviewed our lease proposal.",["The lease includes two parking spaces.","Their secretary requested the cost appendix this morning.","Our proposal uses the new letterhead."],"B","Requesting the cost appendix indirectly indicates that the proposal is under review."],
  ["hard","choice","Should we repair the demonstration unit or replace it entirely?",["The demonstration begins after lunch.","Yes, the unit is in the showroom.","The repair estimate is almost as high as a new one."],"C","The cost comparison provides useful indirect support for choosing replacement."],
  ["hard","statement","Attendance for Saturday's tour has doubled since yesterday.",["Then we may need a second guide.","The tour begins beside the fountain.","Yesterday's weather was pleasant."],"A","Adding a second guide is a logical operational response to doubled attendance."],
  ["hard","offer","Would it be useful if I summarized the survey comments by department?",["The department meeting ended early.","That would help us identify common concerns.","The survey link expires tonight."],"B","Identifying common concerns explains why the offered departmental summary would be useful."],
  ["hard","suggestion","Couldn't we move the product demonstration outdoors?",["The product comes in four colors.","The outdoor sign needs repainting.","Not unless we can protect the equipment from rain."],"C","The weather condition is a natural qualified response to the proposed outdoor move."],
  ["hard","yes_no","Hasn't the legal team approved the revised notice yet?",["They asked us to clarify the cancellation policy.","The notice is posted near the entrance.","Legal documents are kept for seven years."],"A","The request for clarification indirectly shows that final approval has not yet been given."],
];

const letters = ["A", "B", "C"];
const trapCycle = ["wrong_wh_category", "wrong_time", "wrong_location", "same_word_trap", "semantic_association", "wrong_function", "topic_related_but_wrong", "irrelevant_but_plausible", "literal_response"];
const items = authored.map(([difficulty, promptType, prompt, responses, correctAnswer, correctExplanation], index) => {
  const itemId = `toeic-l-p2-${String(index + 26).padStart(4, "0")}`;
  const wrong = letters.filter((letter) => letter !== correctAnswer);
  return {
    itemId,
    itemVersion: 1,
    difficulty,
    promptType,
    prompt,
    responses,
    correctAnswer,
    correctExplanation,
    explanations: Object.fromEntries(wrong.map((letter) => [letter, `Response ${letter} is related to the situation but does not perform the function requested by the prompt.`])),
    listeningFocus: [promptType.replaceAll("_", " "), difficulty === "hard" ? "functional inference" : "appropriate response"],
    usefulPattern: `${promptType.replaceAll("_", " ")} prompt -> contextually appropriate response`,
    tags: [promptType, "workplace_and_daily_life", difficulty],
    distractorTypes: Object.fromEntries(wrong.map((letter, offset) => [letter, trapCycle[(index * 2 + offset) % trapCycle.length]])),
  };
});

bank.items.push(...items);
bank.forms.push({ formId: "toeic-part2-form-b", formVersion: 1, items: items.map((item) => item.itemId) });
fs.writeFileSync(path, `${JSON.stringify(bank, null, 2)}\n`);
