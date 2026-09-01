export const GAMIFICATION_DATA_CHANGED_EVENT = 'english-ai-coach:gamification-data-changed'
export function notifyGamificationDataChanged() { window.dispatchEvent(new Event(GAMIFICATION_DATA_CHANGED_EVENT)) }
