export const LEARNING_DATA_CHANGED_EVENT = 'english-ai-coach:learning-data-changed'

export function notifyLearningDataChanged(): void {
  window.dispatchEvent(new Event(LEARNING_DATA_CHANGED_EVENT))
}
