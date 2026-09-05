import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const bankPath = path.join(root, 'src-tauri/resources/toeic/item-bank-v1/part7.json')
const bank = JSON.parse(fs.readFileSync(bankPath, 'utf8'))
const labels = ['A', 'B', 'C', 'D']

const forms = {
  b: {
    singles: [
      ['email','Updated supplier visit','From: Malcolm Reyes\nTo: Production supervisors\nSubject: Supplier visit\n\nThe representatives from Kanto Components will now visit on Tuesday, October 13, rather than Monday. Their train arrives at 9:20 A.M., and Priya has offered to collect them at Central Station. The factory tour begins at 10:15, followed by lunch in Conference Room B. Please send any technical questions to Malcolm by Friday so they can be added to the agenda.',[
        ['Why was this email written?','To announce a revised supplier visit','To recruit a production supervisor','To cancel a factory tour','To request a new train schedule','The opening sentence changes the supplier visit to Tuesday, October 13.','purpose'],
        ['Who will meet the representatives?','Priya','Malcolm','A train conductor','A technical consultant','The email says that Priya offered to collect them at Central Station.','detail'],
        ['What should supervisors do by Friday?','Send technical questions to Malcolm','Reserve Conference Room A','Confirm the lunch menu','Purchase train tickets','The final sentence asks supervisors to send technical questions by Friday.','next_action']]],
      ['notice','Parking garage maintenance','RIVER PARK OFFICE CENTER\nGARAGE MAINTENANCE\n\nThe west entrance to the parking garage will be closed from 6:00 P.M. Friday until 7:00 A.M. Monday while new lighting is installed. During this period, drivers must enter from King Street and display their regular parking permits. The bicycle area on Level 1 will remain accessible through the pedestrian gate. Deliveries scheduled for Saturday should use the loading zone behind Building C.',[
        ['What is being installed in the garage?','New lighting','Security cameras','A payment machine','Bicycle racks','The notice states that new lighting is being installed.','detail'],
        ['How should drivers enter during the closure?','From King Street','Through the west entrance','Through the pedestrian gate','Behind Building A','Drivers are instructed to enter from King Street during the closure.','detail'],
        ['Where should Saturday deliveries go?','The loading zone behind Building C','Level 1 of the garage','The west entrance','The bicycle area','The final sentence directs Saturday deliveries to the loading zone behind Building C.','detail']]],
      ['advertisement','Orchard Desk plants','ORCHARD DESK PLANTS\nBrighten your workplace with low-maintenance plants delivered in reusable pots. Office packages begin at $35 and include a care guide. Order by noon on Wednesday for delivery the following Friday. Customers within ten kilometers receive free delivery; other locations pay a flat $8 fee. Plants that arrive damaged will be replaced when a photograph is submitted within 48 hours.',[
        ['What is included with an office package?','A care guide','A weekly maintenance visit','A ceramic desk lamp','An extended warranty','The advertisement says every office package includes a care guide.','detail'],
        ['When will a Wednesday morning order be delivered?','The following Friday','The same afternoon','The following Monday','Within ten business days','Orders placed by Wednesday noon are delivered the following Friday.','inference'],
        ['What is required for a damaged plant replacement?','A photograph within 48 hours','The reusable pot in person','An annual membership','An additional delivery fee','The final sentence requires a photograph to be submitted within 48 hours.','detail']]],
      ['article','Market adds evening hours','The Brookdale Farmers Market will remain open until 8:00 P.M. on Thursdays during June and July. Organizers introduced the evening hours after a customer survey showed strong interest from commuters. Six prepared-food vendors have joined the Thursday market, and local musicians will perform near the north entrance from 6:00. The usual Saturday market will continue on its regular morning schedule.',[
        ['Why were evening hours introduced?','A survey showed interest from commuters','Saturday attendance had declined','Musicians requested a later schedule','The north entrance was renovated','The article directly connects the change to a customer survey of commuters.','reason'],
        ['What will begin at 6:00 P.M.?','Musical performances','The Saturday market','A customer survey','Vendor registration','Local musicians will perform near the north entrance from 6:00.','detail'],
        ['What is indicated about the Saturday market?','Its schedule will not change','It will include only prepared food','It will close during June','It will move to the north entrance','The final sentence says the Saturday market continues on its regular morning schedule.','inference']]],
      ['webpage','Crestline conference rooms','CRESTLINE CONFERENCE CENTER\nRoom Atlas seats 20 and includes a wall display. Room Birch seats 12 and has video-conference equipment. Reservations may be changed without charge until two business days before an event. Catering orders must be submitted separately by 3:00 P.M. five business days in advance. Reception staff can arrange visitor badges when a guest list is uploaded.',[
        ['Which room supports video conferences?','Room Birch','Room Atlas','The reception area','The catering room','The webpage says Room Birch has video-conference equipment.','detail'],
        ['When can a room reservation last be changed without charge?','Two business days before the event','On the morning of the event','Five calendar days before the event','After catering is delivered','Changes are free until two business days before an event.','detail'],
        ['How can visitor badges be arranged?','By uploading a guest list','By calling the caterer','By reserving Room Atlas','By arriving before 3:00 P.M.','Reception staff can arrange badges after a guest list is uploaded.','process']]],
      ['memo','Quarterly safety drill','To: All Harbor Street employees\nFrom: Facilities\n\nA building evacuation drill will take place at 2:00 P.M. on September 17. When the alarm sounds, use the nearest marked stairway and meet beside the fountain in Mason Square. Team leaders will check attendance there. Employees assisting customers should guide visitors to the same location. The cafeteria will pause service for approximately twenty minutes during the drill.',[
        ['Where should employees assemble?','Beside the fountain in Mason Square','Inside the cafeteria','At the nearest elevator','On Harbor Street','The memo identifies the fountain in Mason Square as the meeting point.','detail'],
        ['Who will check attendance?','Team leaders','Cafeteria staff','Visiting customers','Facilities contractors','The memo states that team leaders will check attendance.','detail'],
        ['What will temporarily stop?','Cafeteria service','Customer assistance','Stairway access','The alarm system','The cafeteria will pause service for about twenty minutes.','detail']]],
      ['schedule','Mobile clinic timetable','NORTH COUNTY MOBILE CLINIC — WEDNESDAY\nOak Station 8:30–10:00\nRiverside Hall 10:45–12:15\nPine School 1:30–3:00\nHilltop Center 3:45–5:15\n\nAppointments are recommended, but walk-in visitors are accepted when space permits. Bring a photo identification card. The clinic does not operate on public holidays.',[
        ['How long is the clinic at Riverside Hall?','Ninety minutes','Forty-five minutes','Two hours','Three hours','The timetable runs from 10:45 to 12:15, a period of ninety minutes.','inference'],
        ['What should visitors bring?','Photo identification','A school transcript','Proof of an appointment','Cash for admission','The instructions explicitly request a photo identification card.','detail'],
        ['What is indicated about walk-in visitors?','They are accepted if space is available','They are accepted only at Oak Station','They receive priority over appointments','They cannot visit on Wednesdays','The note permits walk-ins when space is available.','inference']]],
      ['chat','Website launch messages','Elena 9:05 A.M.: The product photos are uploaded, but the shipping page still shows last year’s rates.\nDarius 9:08 A.M.: I have the new rate table and can replace it before noon.\nElena 9:10 A.M.: Great. Please message Kai when it is live so he can run the final checkout test.\nDarius 9:12 A.M.: Will do. The launch is still planned for 3:00 P.M.',[
        ['What problem does Elena identify?','Outdated shipping rates are displayed','Product photos are missing','Checkout cannot accept payment','The launch time is unknown','Elena says the shipping page still shows last year’s rates.','detail'],
        ['What will Kai do?','Run a final checkout test','Upload product photographs','Create a new rate table','Approve the launch time','Elena asks that Kai be notified so he can run the final checkout test.','detail'],
        ['When is the website expected to launch?','At 3:00 P.M.','Before noon','At 9:12 A.M.','The following day','Darius confirms that the launch remains planned for 3:00 P.M.','detail']]],
      ['job_posting','Weekend gallery host','WEEKEND GALLERY HOST\nThe Portwell Arts Center is seeking a host for Friday evenings and Sunday afternoons. Responsibilities include welcoming guests, checking tickets, and answering basic questions about current exhibitions. Training is provided during two weekday sessions in November. Applicants must speak clearly and be comfortable using a tablet. Send a résumé and two available interview times by October 22.',[
        ['What is one responsibility of the host?','Checking visitors’ tickets','Selecting artwork for exhibitions','Repairing tablet computers','Teaching weekday classes','Checking tickets is listed among the host’s responsibilities.','detail'],
        ['What will the center provide?','Training','A personal tablet','Friday transportation','An art degree','The posting says training is provided during two weekday sessions.','detail'],
        ['What must applicants submit besides a résumé?','Two available interview times','Two professional photographs','A current exhibition proposal','A November work schedule','Applicants must send two available interview times with their résumé.','detail']]],
      ['instructions','Equipment return procedure','EQUIPMENT RETURN\nPlace cleaned audio equipment in a padded case and attach the completed return label. Bring the case to Service Desk 2 between 8:00 A.M. and 6:00 P.M. A staff member will inspect the equipment and issue a receipt. After-hours returns are not permitted because the outdoor drop box is not designed for electronic devices.',[
        ['What should be attached to the case?','A completed return label','The original sales receipt','A replacement battery','An outdoor drop-box key','The first sentence requires a completed return label.','detail'],
        ['Why are after-hours returns prohibited?','The drop box is unsuitable for electronics','The service desk charges an evening fee','Receipts are issued only in the morning','Padded cases are unavailable after 6:00','The instructions explain that the outdoor drop box is not designed for electronic devices.','reason']]]
    ],
    multiples: [
      ['double','Training registration and manager note',[
        'Document 1 — Registration confirmation\nCourse: Advanced Spreadsheet Tools\nParticipant: Hana Cho\nDate: May 14\nLocation: East Campus, Lab 3\nPlease bring a laptop with the current software installed. Cancellations after May 7 are not refundable.',
        'Document 2 — Manager note\nHana, your client presentation was moved to the morning of May 14. I registered you for the same course on June 6 at West Campus instead. The training office transferred your payment, so no new fee is due. Please confirm that you can attend by Friday.'
      ],[
        ['Where was Hana originally scheduled to attend the course?','East Campus, Lab 3','West Campus, Lab 3','East Campus, Conference Room 6','West Campus, Conference Room 14','Document 1 lists East Campus, Lab 3 as the original location.','detail'],
        ['Why can Hana not attend on May 14?','She has a client presentation','Her laptop software is outdated','The course was canceled','Her manager is traveling','Document 2 says her client presentation was moved to that morning.','cross_document'],
        ['What did the training office do?','Transferred Hana’s payment','Refunded the course in cash','Installed software on her laptop','Moved the client presentation','Document 2 explicitly states that the training office transferred the payment.','detail'],
        ['What is Hana asked to do by Friday?','Confirm attendance on June 6','Cancel the May presentation','Pay a new registration fee','Visit East Campus','The manager asks Hana to confirm that she can attend the June 6 course.','cross_document'],
        ['What is indicated about the June course?','Hana does not need to pay again','It is held in Lab 3 at East Campus','It requires no laptop','It ends before her presentation','The transferred payment in Document 2 means no new fee is due.','inference']]],
      ['double','Repair estimate and customer reply',[
        'Document 1 — Bell Bicycle Repair Estimate\nCustomer: Tomas Reed\nReplace rear tire: $38\nAdjust brakes: $22\nOptional chain replacement: $31\nEstimated completion: Thursday after 4:00 P.M.\nWork begins after customer approval.',
        'Document 2 — Customer reply\nPlease replace the tire and adjust the brakes. I had the chain replaced last month, so do not include that service. I cannot arrive Thursday before closing. Could I collect the bicycle when the shop opens Friday morning? — Tomas'
      ],[
        ['How much will the approved repairs cost?','$60','$31','$38','$91','The approved tire and brake services cost $38 plus $22, totaling $60.','cross_document'],
        ['Which service does Tomas decline?','Chain replacement','Brake adjustment','Rear tire replacement','Safety inspection','Document 2 says not to include the chain replacement.','detail'],
        ['When is the bicycle expected to be ready?','Thursday after 4:00 P.M.','Friday after 4:00 P.M.','Thursday morning','Before the shop closes Wednesday','Document 1 gives Thursday after 4:00 P.M. as the estimated completion.','detail'],
        ['Why does Tomas propose Friday morning?','He cannot arrive before Thursday closing','The repairs begin Friday','The shop charges less on Friday','The replacement tire arrives Friday','Document 2 explains that Tomas cannot arrive Thursday before closing.','cross_document'],
        ['What must happen before repair work begins?','Tomas must approve the work','The chain must be delivered','The bicycle must be collected','The estimate must be reduced','Document 1 says work begins after customer approval, which his reply provides.','process']]],
      ['triple','Conference travel changes',[
        'Document 1 — Rail ticket\nPassenger: Morgan Li\nOutbound: Fairview to Carlton, July 18, 8:10 A.M.\nReturn: Carlton to Fairview, July 20, 6:40 P.M.\nSeat 12A. Changes permitted before departure with a $15 fee.',
        'Document 2 — Conference update\nThe closing session on July 20 has been extended and will finish at 7:00 P.M. A recording will be available to registered participants the next morning.',
        'Document 3 — Morgan’s message\nI cannot make my original return train because of the extended session. I will watch the last hour as a recording and keep the 6:40 ticket. Please do not request a ticket change.'
      ],[
        ['When does Morgan travel to Carlton?','July 18 at 8:10 A.M.','July 18 at 6:40 P.M.','July 20 at 8:10 A.M.','July 20 at 7:00 P.M.','Document 1 lists the outbound trip as July 18 at 8:10 A.M.','detail'],
        ['What changed in the conference schedule?','The closing session will end later','The opening session was canceled','The conference moved to Fairview','The recording became unavailable','Document 2 says the closing session was extended to 7:00 P.M.','detail'],
        ['Why might the original return train be a problem?','It leaves before the closing session ends','It requires a different seat','It departs the next morning','It has a $15 ticket price','Documents 1 and 2 show a 6:40 train and a session ending at 7:00.','cross_document'],
        ['What has Morgan decided to do?','Keep the original return ticket','Pay to move the train to July 21','Miss the outbound train','Attend the full closing session in person','Document 3 says Morgan will keep the 6:40 ticket and watch the last hour later.','cross_document'],
        ['How will Morgan access the missed material?','By watching a recording','By changing seats on the train','By requesting written notes from the railway','By attending a second conference','Documents 2 and 3 establish that a recording will be available and Morgan will watch it.','cross_document']]],
      ['triple','Office furniture delivery',[
        'Document 1 — Purchase order 8841\nClient: Marston Legal\nItems: six adjustable desks, six desk chairs\nRequested delivery: March 11\nDelivery point: fourth-floor reception',
        'Document 2 — Supplier notice\nThe chairs on order 8841 are ready, but two desk frames were damaged in transit. We can deliver all chairs and four desks on March 11, then the remaining desks on March 15. There will be no additional delivery charge.',
        'Document 3 — Client response\nPlease make both deliveries. Our elevator is reserved from 9:00 to 11:00 A.M. on March 11. On March 15, call reception thirty minutes before arrival so building staff can protect the hallway floor.'
      ],[
        ['How many chairs were ordered?','Six','Two','Four','Twelve','Document 1 lists six desk chairs.','detail'],
        ['Why will two desks arrive later?','Their frames were damaged in transit','The elevator is unavailable','The client changed the order','The chairs require assembly','Document 2 attributes the delay to two damaged desk frames.','reason'],
        ['What will be delivered on March 11?','Six chairs and four desks','Four chairs and six desks','Two desks only','The complete original order','Document 2 specifies all six chairs and four desks for the first delivery.','cross_document'],
        ['What should the driver do on March 15?','Call reception thirty minutes ahead','Use the elevator before 9:00','Charge an additional delivery fee','Deliver to the ground-floor lobby','Document 3 asks for a call to reception thirty minutes before arrival.','detail'],
        ['What is indicated about the client?','The client accepts two deliveries','The client canceled the damaged desks','The client will collect the chairs','The client changed the delivery address','Document 3 explicitly approves both deliveries described in Document 2.','cross_document']]],
      ['triple','Community workshop arrangements',[
        'Document 1 — Event listing\nIntroductory Home Repair Workshop\nSaturday, November 9, 1:00–4:00 P.M.\nGray Community Center, Workshop Room\nFee: $20, materials included\nMaximum: 18 participants',
        'Document 2 — Registration update\nAll 18 spaces have been reserved. A waiting list is now open. Registered participants who cannot attend should cancel by November 6 so their place can be offered to someone else.',
        'Document 3 — Instructor email\nPlease remind participants to wear closed-toe shoes. I will arrive at noon to arrange the tools. The center has confirmed that we may use the side entrance next to the loading area.'
      ],[
        ['What does the workshop fee include?','Materials','Transportation','Protective shoes','Lunch','Document 1 states that materials are included in the $20 fee.','detail'],
        ['Why was a waiting list opened?','All available places were reserved','The event fee increased','The instructor will arrive late','The workshop room changed','Document 2 says all 18 spaces have been reserved.','reason'],
        ['By when should an unavailable participant cancel?','November 6','November 9 at noon','Before the waiting list opens','After the workshop ends','Document 2 gives November 6 as the cancellation deadline.','detail'],
        ['What should participants wear?','Closed-toe shoes','A name badge from the loading area','Formal business clothing','A tool belt supplied by the center','Document 3 instructs participants to wear closed-toe shoes.','detail'],
        ['What is indicated about the instructor?','The instructor will arrive one hour early','The instructor is on the waiting list','The instructor will collect the $20 fee','The instructor changed the participant limit','Documents 1 and 3 show a 1:00 start and a noon arrival.','cross_document']]]
    ]
  },
  c: {
    singles: [
      ['email','Catalog proof approval','From: Aisha Grant\nTo: Ren Ito\nSubject: Spring catalog proof\n\nThe revised catalog proof looks good. The page numbers are now correct, and the photograph on page 12 is much clearer. Please ask the printer to produce 800 copies rather than 600 because two additional stores will participate in the promotion. If the quantity change affects Friday’s delivery date, let me know before you approve the final order.',[
        ['What does Aisha ask Ren to change?','The number of catalog copies','The photograph on page 12','The page numbering style','The Friday promotion date','Aisha requests 800 copies instead of 600.','detail'],
        ['Why are more copies needed?','Two more stores joined the promotion','The original copies were unclear','The printer reduced its price','The catalog gained more pages','The email links the larger quantity to two additional participating stores.','reason'],
        ['When should Ren contact Aisha?','If the change affects Friday’s delivery','After all catalogs are distributed','Before correcting page numbers','When the promotion ends','Aisha requests notice if the quantity change affects the Friday delivery date.','next_action']]],
      ['notice','Water service interruption','CEDAR BUSINESS PARK\nWATER SERVICE NOTICE\n\nWater will be unavailable in Buildings 2 and 3 from 7:30 to 9:30 A.M. on Tuesday while a damaged valve is replaced. Restrooms in Building 1 will remain open. The café will begin service at 10:00 instead of 8:00, but the lobby coffee kiosk will operate normally. Building managers should turn off water-connected equipment before leaving Monday.',[
        ['Why will water service stop?','A damaged valve will be replaced','The café is being renovated','Building 1 will close','A coffee kiosk is opening','The notice states that workers will replace a damaged valve.','reason'],
        ['Which facility remains available during the interruption?','Restrooms in Building 1','The café at 8:00','Water equipment in Building 3','The Building 2 kitchen','The notice specifically says Building 1 restrooms remain open.','detail'],
        ['What should managers do on Monday?','Turn off equipment connected to water','Move the lobby coffee kiosk','Open the café two hours early','Inspect Building 1 restrooms','The final sentence directs managers to turn off water-connected equipment.','next_action']]],
      ['advertisement','CityCycle membership','CITYCYCLE ANNUAL MEMBERSHIP\nRide from any of our 45 bicycle stations for $72 per year. The first 45 minutes of every trip are included; longer trips incur usage charges. Members receive a helmet discount at three partner shops and may reserve a bicycle up to 20 minutes before pickup. Join online before April 30 to receive an extra month at no cost.',[
        ['What is included in the annual fee?','The first 45 minutes of each trip','Unlimited trips of any length','A free helmet','Delivery of a bicycle','The advertisement includes the first 45 minutes of every trip.','detail'],
        ['How early may members reserve a bicycle?','Twenty minutes before pickup','Forty-five minutes before pickup','One month before pickup','Only after arriving at a station','Members may reserve a bicycle up to twenty minutes in advance.','detail'],
        ['What is offered to people who join by April 30?','One additional month free','A membership for $45','Free usage at partner shops','Access to 72 stations','The final sentence offers an extra month at no cost.','detail']]],
      ['article','Bakery reuses heat','The Larch Street Bakery has installed a system that captures heat from its ovens and uses it to warm water for cleaning. Owner Sofia Mendes says the change has reduced monthly gas use by nearly 14 percent. The equipment was partly funded by a city efficiency grant, and installation took only three days. The bakery plans to share its first-year results with other small food businesses next spring.',[
        ['How does the bakery reuse oven heat?','It warms water for cleaning','It powers delivery vehicles','It cools the sales area','It dries packaged bread','The article says captured heat is used to warm cleaning water.','detail'],
        ['What helped pay for the equipment?','A city efficiency grant','Higher bread prices','A gas company refund','Other food businesses','The equipment was partly funded by a city efficiency grant.','detail'],
        ['What does the bakery plan to do next spring?','Share its first-year results','Replace all of its ovens','Apply for its first grant','Close for a three-day installation','The last sentence says results will be shared with other small food businesses.','next_action']]],
      ['webpage','Lakeview coworking passes','LAKEVIEW WORK HUB\nDay passes provide a desk, wireless Internet, and unlimited tea from 8:00 A.M. to 6:00 P.M. Meeting rooms are charged separately and require online reservations. Monthly members may enter from 6:00 A.M. and receive five meeting-room hours. Lockers are available to either group for $12 per month. First-time visitors must show identification at reception.',[
        ['What is included in a day pass?','Unlimited tea','A monthly locker','Five meeting-room hours','Entry from 6:00 A.M.','The webpage includes unlimited tea with a day pass.','detail'],
        ['Who receives five meeting-room hours?','Monthly members','First-time visitors','Day-pass users','Reception employees','Five meeting-room hours are a monthly-member benefit.','detail'],
        ['What must first-time visitors do?','Show identification','Reserve a locker','Arrive before 8:00 A.M.','Purchase a monthly membership','The final sentence requires identification at reception.','process']]],
      ['memo','Archive relocation','To: Research staff\nFrom: Records Department\n\nThe historical sales archive will move from the basement to Room 508 next week. Paper files will be unavailable Tuesday and Wednesday while shelves are installed and boxes are transferred. Scanned records can still be accessed through the internal portal. Requests submitted after 3:00 P.M. Monday will be handled on Thursday. Contact Mei Santos if an urgent legal request cannot wait.',[
        ['When will paper files be unavailable?','Tuesday and Wednesday','Monday morning only','Thursday and Friday','All of next month','The memo states paper files will be unavailable Tuesday and Wednesday.','detail'],
        ['What remains accessible during the move?','Scanned records','Basement shelves','All paper files','Room 508 boxes','Scanned records remain accessible through the internal portal.','detail'],
        ['Who should be contacted about an urgent legal request?','Mei Santos','Research staff','A sales manager','The building installer','The final sentence names Mei Santos as the contact.','detail']]],
      ['schedule','Airport shuttle service','HORIZON HOTEL AIRPORT SHUTTLE\nHotel departures: 5:30, 7:00, 8:30, 10:00 A.M.\nAirport departures: 6:15, 7:45, 9:15, 10:45 A.M.\nTravel time is approximately 25 minutes. Reserve at reception at least one hour ahead. Each passenger may bring one large suitcase; additional luggage costs $6 per item.',[
        ['What time does the second shuttle leave the hotel?','7:00 A.M.','6:15 A.M.','7:45 A.M.','8:30 A.M.','The hotel departure list shows 7:00 A.M. as the second shuttle.','detail'],
        ['How far in advance should a seat be reserved?','At least one hour','Twenty-five minutes','Six hours','One day','The schedule directs guests to reserve at least one hour ahead.','detail'],
        ['When is an extra luggage fee charged?','For more than one large suitcase','For every suitcase','For airport departures only','For trips longer than 25 minutes','One large suitcase is allowed, and additional items cost $6 each.','inference']]],
      ['chat','Catering order messages','Noah 11:20 A.M.: We have 26 people confirmed for Thursday’s lunch, including four vegetarians.\nMina 11:23 A.M.: The Garden Café can add four vegetable wraps, but they need the final order by 2:00 today.\nNoah 11:25 A.M.: Please add them. I will update the purchase request from 22 to 26 meals.\nMina 11:27 A.M.: Done. Delivery remains scheduled for 11:45 Thursday.',[
        ['How many vegetarian meals are needed?','Four','Twenty-two','Twenty-six','Two','Noah says four of the confirmed attendees are vegetarians.','detail'],
        ['What will Noah update?','The purchase request','The delivery time','The café menu','The lunch date','Noah says he will update the purchase request to 26 meals.','detail'],
        ['When will the lunch be delivered?','11:45 on Thursday','2:00 on Thursday','11:20 today','11:45 today','Mina confirms that delivery remains at 11:45 Thursday.','detail']]],
      ['job_posting','Seasonal visitor guide','SEASONAL VISITOR GUIDE\nPine Coast Nature Reserve needs guides from June through August. Guides lead 45-minute walking tours, answer questions, and record daily visitor totals. Weekend availability and basic first-aid certification are required. Knowledge of local birds is desirable. Interviews will be held online during the week of April 8. Apply through the reserve website by March 29.',[
        ['What must applicants have?','Basic first-aid certification','Professional photography equipment','Advanced accounting experience','A vehicle for online interviews','First-aid certification is explicitly required.','detail'],
        ['What is preferred but not required?','Knowledge of local birds','Weekend availability','Ability to count visitors','First-aid certification','Knowledge of local birds is described as desirable rather than required.','inference'],
        ['How will interviews be conducted?','Online','At the nature reserve','During walking tours','By telephone every weekend','The posting states that interviews will be held online.','detail']]],
      ['instructions','Shared printer setup','SHARED PRINTER SETUP\nConnect your computer to the Staff-Secure network. Open the Devices panel and select Ridge-Color-4 from the available printers. Enter your employee number when prompted, then print the one-page test sheet. If the sheet does not appear within two minutes, leave the printer on and contact the technology desk; do not repeat the installation.',[
        ['Which printer should employees select?','Ridge-Color-4','Staff-Secure','Devices-2','Technology-1','The instructions name Ridge-Color-4 as the printer.','detail'],
        ['What should an employee do if the test sheet does not appear?','Contact the technology desk','Repeat the installation immediately','Turn off the printer','Change the employee number','The final sentence directs the employee to leave the printer on and contact the technology desk.','process']]]
    ],
    multiples: [
      ['double','Venue invoice and coordinator email',[
        'Document 1 — Brook Hall Invoice\nClient: Westline Association\nEvent date: February 21\nMain hall rental: $900\nSound system: $120\nCoffee service for 60: $180\nPayment due: February 7',
        'Document 2 — Coordinator email\nAttendance has risen to 75. Brook Hall can provide coffee for the additional guests for $45, but we will bring our own portable speakers, so please remove the sound system. The event date and room remain unchanged.'
      ],[
        ['What was the original total of the invoice?','$1,200','$1,020','$1,080','$900','Document 1 totals $900, $120, and $180, which equals $1,200.','inference'],
        ['Why is more coffee needed?','Attendance increased','The event date changed','The sound system was removed','Payment is overdue','Document 2 explains that attendance rose from 60 to 75.','cross_document'],
        ['Which charge should be removed?','The sound system','The main hall','Coffee service','The additional guest fee','The coordinator says the association will bring speakers and asks to remove the sound system.','detail'],
        ['What remains unchanged?','The event date and room','The number of attendees','The final invoice total','The source of the speakers','Document 2 explicitly says the event date and room remain unchanged.','detail'],
        ['What will the revised total be?','$1,125','$1,245','$1,065','$945','Removing $120 and adding $45 changes the $1,200 original total to $1,125.','cross_document']]],
      ['double','Book order and library reply',[
        'Document 1 — Meridian Books Order\nOrder 5307\nRiver Ecology, 8 copies at $28\nUrban Trees, 5 copies at $24\nStandard shipping: $18\nExpected dispatch: January 16',
        'Document 2 — Library reply\nPlease reduce River Ecology to six copies. Keep all five copies of Urban Trees. We need the books for a January 24 seminar, so the expected dispatch date is acceptable. Charge the order to our existing institutional account.'
      ],[
        ['How many copies of River Ecology were first ordered?','Eight','Six','Five','Twenty-eight','Document 1 lists eight copies of River Ecology.','detail'],
        ['Which quantity does the library leave unchanged?','Five copies of Urban Trees','Eight copies of River Ecology','Six copies of Urban Trees','Twenty-four copies of River Ecology','Document 2 keeps all five Urban Trees copies.','cross_document'],
        ['Why does the library mention January 24?','A seminar will use the books','The account expires that day','Shipping begins that day','The order must be reduced that day','Document 2 identifies January 24 as the seminar date.','reason'],
        ['What is indicated about the dispatch date?','The library accepts it','It must be moved earlier','It was changed to January 24','It depends on a new account','The reply explicitly says the expected dispatch date is acceptable.','inference'],
        ['How much will the books cost before shipping after the change?','$288','$344','$306','$270','Six books at $28 plus five at $24 equals $288.','cross_document']]],
      ['triple','Museum group visit',[
        'Document 1 — Group booking\nOrganization: Delmar Language School\nDate: April 16\nArrival: 10:30 A.M.\nVisitors: 24 students, 3 teachers\nProgram: Architecture tour at 11:00\nLunch room reserved: 12:15–1:00',
        'Document 2 — Museum notice\nThe east lobby will be closed April 15–17 for floor repairs. Groups should enter through the sculpture garden and check in at the temporary desk. Buses may unload on Palmer Road.',
        'Document 3 — Teacher message\nOur bus company has confirmed a 10:20 arrival on Palmer Road. I will take the attendance list to the temporary desk while the other teachers lead students through the sculpture garden entrance.'
      ],[
        ['How many people are in the school group?','Twenty-seven','Twenty-four','Three','Twenty-one','Document 1 lists 24 students plus 3 teachers, totaling 27.','inference'],
        ['Why can the group not use the east lobby?','The floor is being repaired','A tour begins there','The bus will arrive late','Lunch is being served there','Document 2 says the east lobby is closed for floor repairs.','reason'],
        ['Where will the bus unload?','Palmer Road','The east lobby','The sculpture garden','The lunch room','Documents 2 and 3 identify Palmer Road as the bus unloading and arrival point.','cross_document'],
        ['What will one teacher bring to check-in?','The attendance list','The lunch reservation','Floor repair equipment','Architecture drawings','Document 3 says the teacher will take the attendance list to the temporary desk.','detail'],
        ['How long after the planned bus arrival does the tour begin?','Forty minutes','Ten minutes','One hour','One hour and forty minutes','Documents 1 and 3 show arrival at 10:20 and the tour at 11:00.','cross_document']]],
      ['triple','Software license renewal',[
        'Document 1 — Renewal quote Q-332\nClient: Alden Design\n15 StudioPro licenses\nAnnual price per license: $96\nRenewal date: December 1\nQuote valid through November 20',
        'Document 2 — Department message\nTwo interns finish on November 28, and their licenses will not be needed afterward. A new designer starts December 4 and will require access on the first day. Please determine the correct renewal quantity.',
        'Document 3 — Purchasing response\nI will renew 14 licenses: 15 current users minus the two interns, plus one for the new designer. The supplier confirmed the new account can be activated on December 4 without changing the annual price.'
      ],[
        ['When does the current license term renew?','December 1','November 20','November 28','December 4','Document 1 gives December 1 as the renewal date.','detail'],
        ['Why are two licenses no longer needed?','Two interns are leaving','The annual price increased','The quote expires','The design department is closing','Document 2 says two interns finish on November 28.','reason'],
        ['How many licenses will purchasing renew?','Fourteen','Fifteen','Thirteen','Sixteen','Document 3 states the calculated renewal quantity is 14.','cross_document'],
        ['When does the new designer need access?','December 4','December 1','November 20','November 28','Documents 2 and 3 identify December 4 as the start and activation date.','cross_document'],
        ['What is indicated about the price?','It remains $96 per license','It falls after November 20','It includes two free intern accounts','It changes when the new account is activated','Documents 1 and 3 show the $96 annual price remains unchanged.','cross_document']]],
      ['triple','Restaurant equipment service',[
        'Document 1 — Service request\nRiva Restaurant reports that refrigerator unit 2 is not maintaining temperature. Preferred visit: Monday morning before 10:30. Kitchen opens for lunch preparation at 11:00.',
        'Document 2 — Technician schedule\nMonday: 8:00 Grove Café; 9:45 Riva Restaurant; 11:30 North Hotel\nTechnician: L. Patel\nParts carried: sensors, door seals, control panels',
        'Document 3 — Service report\nArrived at Riva at 9:40. A loose temperature sensor was reconnected and tested. Unit 2 reached the required temperature at 10:15. No replacement parts were used. Follow-up inspection recommended in three months.'
      ],[
        ['What problem did Riva report?','A refrigerator was too warm','Lunch preparation began too early','A door seal was missing','The technician arrived late','Document 1 reports that unit 2 was not maintaining temperature.','detail'],
        ['Who was assigned to the visit?','L. Patel','A Grove Café employee','A North Hotel manager','The lunch chef','Document 2 names L. Patel as the technician.','detail'],
        ['Did the technician meet the requested arrival window?','Yes, arrival was before 10:30','No, arrival was after 11:00','No, arrival was at 11:30','The documents do not give an arrival time','Documents 1 and 3 show a requested time before 10:30 and arrival at 9:40.','cross_document'],
        ['How was the problem resolved?','A sensor was reconnected','A control panel was replaced','A new refrigerator was installed','A door seal was purchased','Document 3 says a loose temperature sensor was reconnected.','detail'],
        ['What should happen in three months?','A follow-up inspection','A replacement-part delivery','A change to lunch hours','A visit to North Hotel','The service report recommends a follow-up inspection in three months.','next_action']]]
    ]
  }
}

function makeQuestion(formKey, index, spec) {
  const [prompt, correct, ...rest] = spec
  const distractors = rest.slice(0, 3)
  const evidence = rest[3]
  const skill = rest[4]
  const correctIndex = (index + (formKey === 'c' ? 2 : 0)) % 4
  const choices = []
  let wrongIndex = 0
  for (let i = 0; i < 4; i++) {
    choices.push({ choice: labels[i], text: i === correctIndex ? correct : distractors[wrongIndex++] })
  }
  const correctAnswer = labels[correctIndex]
  return {
    itemId: `toeic-r-p7-${formKey}-${String(index + 1).padStart(3, '0')}`,
    itemVersion: 1,
    publicationState: 'published',
    questionType: skill,
    blankId: prompt,
    choices,
    correctAnswer,
    correctExplanation: `${evidence} Therefore, “${correct}” is the only choice supported by the text.`,
    distractorExplanations: Object.fromEntries(labels.filter((x) => x !== correctAnswer).map((x) => [x, `Choice ${x} conflicts with the document's stated time, quantity, location, purpose, or required action.`])),
    completedContext: `Evidence: ${evidence}`,
    skillCategory: skill,
    difficulty: skill === 'cross_document' || skill === 'inference' ? 'hard' : index % 5 === 0 ? 'easy' : 'medium',
    usefulPattern: 'Locate the exact evidence first, then reject choices that alter a date, quantity, place, purpose, or action.',
    extraExample: null
  }
}

function buildForm(formKey) {
  const source = forms[formKey]
  let qIndex = 0
  const sets = source.singles.map((entry, i) => {
    const [documentType, title, passage, questions] = entry
    return {
      textSetId: `toeic-r-p7-set-s${String(i + 1).padStart(2, '0')}-${formKey}`,
      version: 1,
      publicationState: 'published',
      documentType,
      title,
      passage,
      difficulty: i < 3 ? 'easy' : i < 8 ? 'medium' : 'hard',
      domain: 'workplace_and_daily_life',
      questions: questions.map((q) => makeQuestion(formKey, qIndex++, q))
    }
  })
  sets.push(...source.multiples.map((entry, i) => {
    const [kind, title, documents, questions] = entry
    return {
      textSetId: `toeic-r-p7-set-m${String(i + 1).padStart(2, '0')}-${formKey}`,
      version: 1,
      publicationState: 'published',
      documentType: 'multiple_documents',
      title,
      passage: documents.join('\n\n----------\n\n'),
      difficulty: kind === 'double' ? 'medium' : 'hard',
      domain: 'workplace_and_daily_life',
      questions: questions.map((q) => makeQuestion(formKey, qIndex++, q))
    }
  }))
  if (qIndex !== 54) throw new Error(`Form ${formKey} generated ${qIndex} questions`)
  return sets
}

for (const formKey of ['b', 'c']) {
  const formId = `toeic-part7-form-${formKey}`
  const generated = buildForm(formKey)
  bank.forms = bank.forms.filter((f) => f.formId !== formId)
  const ids = new Set(generated.map((s) => s.textSetId))
  bank.textSets = bank.textSets.filter((s) => !ids.has(s.textSetId))
  bank.forms.push({
    formId,
    formVersion: 1,
    title: `Part 7 · Form ${formKey.toUpperCase()}`,
    publicationState: 'published',
    textSetIds: generated.map((s) => s.textSetId)
  })
  bank.textSets.push(...generated)
}

fs.writeFileSync(bankPath, `${JSON.stringify(bank, null, 2)}\n`)
console.log(`Part 7 now contains ${bank.forms.length} forms, ${bank.textSets.length} sets, and ${bank.textSets.reduce((n, s) => n + s.questions.length, 0)} questions.`)
