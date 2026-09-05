export const REVIEW_DATA_CHANGED_EVENT='english-ai-coach:review-data-changed'
export function notifyReviewDataChanged(){window.dispatchEvent(new Event(REVIEW_DATA_CHANGED_EVENT))}
