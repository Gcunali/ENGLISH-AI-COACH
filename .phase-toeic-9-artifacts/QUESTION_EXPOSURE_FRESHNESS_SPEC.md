# Question Exposure and Freshness Specification

- Identity is `itemId:itemVersion`.
- Exposure is created only by rows in `toeic_answer`, therefore preview, review, validators and tests do not count in the human database.
- The dashboard reports unique seen, unseen, total scored answers and repeat exposure.
- Personalized practice selects the least-exposed available form family and freezes exact item IDs before presentation.
- Resume reopens that frozen snapshot. Full simulations remain fixed A/B/C compositions and are never dynamically personalized.
