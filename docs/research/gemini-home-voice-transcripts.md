# Research: Gemini for Home voice requests as a grocery intake

Checked 2026-11-16. Question: now that "Hey Google" requests to Nest speakers
appear as chats in the Gemini Android app, can this app read them, so that
"Hey Google, add milk to the shopping list" reaches our intake?

Labels used below: **[primary]** = Google help, developer docs or blog.
**[secondary]** = news, forums or Reddit, used only for recent rollout facts.
**[unconfirmed]** = I could not find a source that settles it.

## TL;DR

- No public API returns Gemini app chats or Home voice history. The Data
  Portability API has no Gemini, Assistant or Home resource group. It is also
  limited to EEA, Switzerland and UK accounts, so it is not available in
  Australia.
- On Gemini for Home, the voice shopping list goes to **Google Keep**, and
  reminders and tasks go to **Google Tasks**. Google Tasks is the only one of
  these with a consumer REST API, and this app already polls it.
- Best route: say "Hey Google, remind me to buy milk" (or "add milk to my
  tasks") and let the existing Tasks poller pick it up. Test this on a real
  account first. See the risks in section 4.
- I found no official way to silence Gemini's spoken reply.

## 1. What changed

**Gemini replaced Google Assistant on speakers and displays.**
- Google announced Gemini for Home on 2025-10-01. It "not only replaces the
  Google Assistant on your smart displays and speakers" and started "this month
  in early access" [primary]
  (https://blog.google/products-and-platforms/devices/google-nest/next-era-gemini-google-home-launch).
- Once a home switches, "compatible devices in your home ... always use Gemini
  for Home voice assistant", and the home "can't switch back to Google
  Assistant" [primary]
  (https://support.google.com/googlehome/answer/16618650?hl=en).
- **Australia** is in the early-access country list, with English as a
  supported language. The same list includes the US, UK, NZ, Canada and others
  [primary] (https://support.google.com/googlehome/answer/16618650?hl=en).
- The free tier covers "calendars, notes, lists, and reminders". Gemini Live
  and some other features need Google Home Premium [primary]
  (https://support.google.com/googlehome/answer/16618650?hl=en).

**Where Home conversations are stored, according to Google's docs.**
- "Your Gemini for Home activity, such as voice assistant queries, saves to
  your Home History in My Activity" (`myactivity.google.com/product/home`). It
  is kept for 18 months by default, and needs Voice Match plus Home History
  turned on [primary]
  (https://support.google.com/googlehome/answer/16613583?hl=en).
- Google's marketing footnote says "Gemini features work independently of
  Gemini apps" [primary]
  (https://home.google.com/gemini-for-home-voice-assistant).
- The Gemini Apps Privacy Hub describes Home History and Gemini Apps "Keep
  Activity" as separate settings [primary]
  (https://support.google.com/gemini/answer/13594961?hl=en).

**What users report now [secondary].** A Pixel 10 Pro owner wrote about 9 days
ago (around 2026-11-07) that "every time I use a voice command through my
phone, my car's Android Auto, or my Google Home speaker, it now creates an
unnecessary chat in my Gemini app". They had only seen this "for the past three
or four days". A reply says there is no way to turn it off
(https://www.reddit.com/r/googleassistant/comments/1xqk0pc/every_time_i_speak_with_gemini_now_it_initiates_a).
This matches what the user sees.

**[unconfirmed]** I found no Google announcement or release note saying that
Home speaker requests now sync into the Gemini app history. I also found
nothing that explains why the chats show on Android but not on
gemini.google.com. One guess: the Android app shows device-originated
("Hey Google") chats that the web history filters out. I could not verify this.
Google's own docs (above) still say Home activity goes to Home History. Treat
the behaviour as unannounced and liable to change.

**Correction to our own specs.** `FEATURE_GOOGLE_TASKS.md` and
`FEATURE_VOICE.md` say voice shopping items go to "Google's own shopping list
(Google Home / Express)". That is out of date:
- "Your Shopping List and Assistant Notes and Lists are now saved in Google
  Keep", and "Google Shopping List will redirect to Google Keep" [primary]
  (https://support.google.com/assistant/answer/14171370?hl=en).
- Gemini for Home's supported services list "Google Keep - Create notes &
  lists" and "Google Tasks - Create reminders" [primary]
  (https://support.google.com/googlehome/answer/16709732?hl=en).
- The Notes & lists provider setting for Gemini for Home names Google Keep
  only [primary] (https://support.google.com/googlehome/answer/16722557?hl=en).
- Third-party list providers such as Bring! are not offered under Gemini for
  Home [secondary, product-expert forum reply]
  (https://support.google.com/assistant/thread/464203449/can-we-get-lists-like-bring-back-now-that-we-are-using-gemini?hl=en).

## 2. Can the transcripts be read by a program?

| Route | Result | Source |
|---|---|---|
| Gemini API (ai.google.dev) | **No.** It is a model API. I found no endpoint for consumer Gemini app chats. The only documented export of Gemini Apps Activity is Takeout (My Activity → "Gemini Apps"). | [primary] https://support.google.com/gemini/answer/16920332?hl=en |
| My Activity / Home History | **No API.** You can only view and delete it in the browser or the Home app. | [primary] https://support.google.com/googlehome/answer/16613583?hl=en |
| Data Portability API | **No.** See below. | below |
| Google Takeout | Manual. A one-off export, or "every 2 months for one year". Far too slow for a grocery list. | [primary] https://support.google.com/accounts/answer/3024190?hl=en |
| Google Home APIs / automations | **Fixed phrases only, and no webhook.** See below. | below |
| Google Keep API | **Not for consumer accounts.** It is "used to create apps allowing enterprise administrators to manage Google Keep notes". | [primary] https://developers.google.com/workspace/keep/api/guides |
| Google Tasks API | **Yes.** It is already integrated (`FEATURE_GOOGLE_TASKS.md`), and Gemini for Home writes to Tasks by voice. | [primary] https://support.google.com/googlehome/answer/16722329?hl=en |

### Data Portability API
- The `myactivity.*` resource groups are `maps`, `myadcenter`, `play`,
  `search`, `shopping` and `youtube`. The scope list has no Gemini, Assistant,
  Home, Keep or Tasks group [primary]
  (https://developers.google.com/data-portability/user-guide/scopes,
  https://developers.google.com/data-portability/schema-reference/my_activity).
  `myactivity.shopping` is Google Shopping *activity* (searches and ad
  interactions), not the shopping list.
- **Availability:** EU/EEA countries, Switzerland and the United Kingdom.
  **Australia is not listed** [primary]
  (https://support.google.com/accounts/answer/14452558?hl=en).
- **Access modes:** the user picks "one-time access ... or time-based access
  for 30 or 180 days". With time-based access "you can export resources every
  24 hours until the consent expires" [primary]
  (https://developers.google.com/data-portability/user-guide/introduction).
  An archive takes "minutes to hours" to build [primary] (same page).
- Using it in production requires app verification: brand, use case, demo
  video, and a security assessment for restricted scopes [primary]
  (https://developers.google.com/data-portability/user-guide/overview).
- Verdict: not usable here. It is the wrong region, it has no matching
  resource group, and a 24-hour cadence is too slow.

### Google Home automations and Home APIs
- The script editor has a voice starter, `assistant.event.OkGoogle`, with a
  `query` field. It matches with `is` / `isNot`, which means equality with a
  fixed phrase, for example `is: Movie time` [primary]
  (https://developers.home.google.com/automations/schema/reference/entity/assistant/ok_google_event).
- The available actions are device commands, `assistant.command.Broadcast`,
  `assistant.command.OkGoogle` (speak a command to a speaker) and
  `home.command.Notification` (a push notification to home members). The list
  has **no HTTP or webhook action**, and I found no way to pass the spoken text
  into an action [primary]
  (https://developers.home.google.com/automations/starters-conditions-and-actions).
- Google says it "improved reliability of triggering user-created automations
  by voice" on Gemini for Home, so custom-phrase starters still work [primary]
  (https://support.google.com/googlehome/answer/15962877?hl=en).
- The Home APIs are Android and iOS SDKs (Kotlin DSL). Limits include 64
  automations per structure and 128 executions per developer per structure per
  day [primary] (https://developers.home.google.com/apis/android/automation).
- The only way this could reach our server: one automation per fixed phrase
  ("Hey Google, we need milk") that switches a virtual device exposed by a
  Cloud-to-cloud integration we host. Our fulfillment would receive the
  EXECUTE call. That brings back the Smart Home Action / OAuth account-linking
  design that AGENTS.md dropped, and it only knows a fixed vocabulary.
  **[unconfirmed]** I did not verify Cloud-to-cloud test-mode rules for this.

### Can Gemini on a Home speaker write to Keep or Tasks now?
- **Keep:** yes. Example: "Add the ingredients to make an authentic vegetarian
  Pad Thai to my shopping list" [primary]
  (https://support.google.com/googlehome/answer/16618650?hl=en). It needs
  Personal results and Keep set as the Notes & lists provider [primary]
  (https://support.google.com/googlehome/answer/16722557?hl=en). Some users
  report Keep shopping-list commands failing during the rollout [secondary]
  (https://tech.yahoo.com/ai/gemini/articles/google-home-bug-disrupts-keep-195000605.html).
- **Tasks:** yes. Examples: "Set a task", "Remind me to call Mom at noon
  tomorrow", and "change my task to buy milk to 5:00 PM". It needs Voice Match
  and Personal results [primary]
  (https://support.google.com/googlehome/answer/16722329?hl=en).
- **[unconfirmed]** Which Tasks list a spoken task lands in (probably the
  default "My Tasks"), and whether you can name another list by voice ("add
  milk to my Groceries list in Tasks"). Also unconfirmed: whether the title is
  "buy milk" or "milk", and whether the task carries a due date.

## 3. Can the spoken reply be suppressed?

- **No official setting found.** The Gemini for Home settings cover voice,
  language, "Hey Google" sensitivity, response filtering and Continued
  Conversation. None of them is "no spoken reply" [primary]
  (https://support.google.com/googlehome/answer/16618650?hl=en).
- The 2026-10-14 release notes say Gemini is "staying silent and closing the
  microphone immediately when you give a direct command to stop talking".
  That only ends a conversation; it does not mute confirmations [secondary,
  quoting the release notes]
  (https://www.androidpolice.com/gemini-for-home-now-knows-when-to-stop-talking).
- Two community claims, not documented by Google [secondary]:
  - A speaker in the same room as the device it controls plays a chime instead
    of speaking.
  - Routines and scripted automations run without the usual spoken
    confirmation.

  Sources: https://support.google.com/gemini/thread/389366810/how-can-i-stop-gemini-from-telling-me-it-has-done-what-i-asked?hl=en,
  https://community.home-assistant.io/t/google-home-scripted-voice-commands-no-longer-working-with-speakers-only-working-with-assistant-on-phone/953713.
  Neither helps with list or task adds.
- **Unofficial approaches** (muting the speaker volume, or intercepting Cast
  traffic) are fragile. Anything that automates Google's web UI or scrapes My
  Activity or the Gemini app breaks Google's Terms of Service
  (https://policies.google.com/terms) and can get the account locked. Not
  recommended.

**Region (Australia):** Gemini for Home is available, since AU is in the
early-access list [primary]
(https://support.google.com/googlehome/answer/16618650?hl=en). The Data
Portability API is not [primary]
(https://support.google.com/accounts/answer/14452558?hl=en). Google says
supported services "are dependent on region" [primary]
(https://support.google.com/googlehome/answer/16709732?hl=en).
**[unconfirmed]** I did not confirm that the Tasks and Keep voice actions work
in AU specifically.

## 4. Recommendations, ranked

1. **Speak into Google Tasks and reuse the existing poller.**
   Effort: low. Risk: low to medium.
   - Say "Hey Google, remind me to buy milk" or "add milk to my tasks". On the
     Groceries settings page (`/settings/help`), choose the list the voice
     tasks land in.
   - Code changes:
     - Teach `services/google_tasks/title.rs` to strip "buy " and "get ".
     - Ignore due dates.
     - Update `FEATURE_GOOGLE_TASKS.md` and `FEATURE_VOICE.md`, which still
       say voice does not reach Tasks.
   - Risk: the poller records **and deletes every open task** in the chosen
     list. If voice tasks land in "My Tasks" and the household keeps
     non-grocery tasks there, those tasks would end up in Pending and be
     deleted from Google. Mitigations:
     - confirm a dedicated list can be targeted by voice; or
     - give the speaker's Voice Match a dedicated Google account; or
     - add a keyword filter.
   - Other risk: the reminder nags the phone if it has a due time.
   - First step: a manual test on a real account, noting which list, the
     title, the due date and the notes.
2. **Keep the Alexa bridge as the reliable voice channel.** Effort: none,
   since it is already built (`sidecars/alexa-bridge`). Risk: low.
3. **Google Keep shopping list.** Effort: medium. Risk: high.
   - "Add milk to my shopping list" works natively by voice and lands in Keep.
   - The official Keep API is enterprise-admin only. Reading a consumer
     account needs `gkeepapi` (unofficial, master-token auth, Python), which
     AGENTS.md already lists as a paused sidecar.
   - It can break without notice, and the ToS risk is real.
4. **Fixed-phrase Home automations into a Cloud-to-cloud virtual device.**
   Effort: high (public HTTPS fulfillment, OAuth account linking, one
   automation per item, at most 64). Risk: medium. Only worth it for a handful
   of staples. It still has no free text.
5. **Data Portability API polling.** Not possible: there is no Gemini, Home or
   Assistant resource group, it does not cover Australia, and it runs at most
   every 24 hours.
6. **Takeout.** Not viable: it is manual or runs every 2 months.
7. **Scraping My Activity, Home History or Gemini app chats.** Not
   recommended. There is no API, it breaks the ToS, it involves 2FA and
   bot-detection, and it would break with each UI change.

## Open questions to test by hand
- Which Google Tasks list does a Gemini for Home voice task use, and can you
  name another list by voice?
- What title and due date does "remind me to buy milk" produce?
- Do Home speaker chats still appear in the Gemini Android app next month?
  The behaviour is unannounced.
