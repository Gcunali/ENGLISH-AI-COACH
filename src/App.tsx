import { Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import { DashboardPage } from './pages/DashboardPage'
import { HistoryPage } from './pages/HistoryPage'
import { LessonDetailsPage } from './pages/LessonDetailsPage'
import { NewLessonPage } from './pages/NewLessonPage'
import { SettingsPage } from './pages/SettingsPage'
import { ProgressPage } from './pages/ProgressPage'
import { PlacementPage } from './pages/PlacementPage'
import { PlacementResultPage } from './pages/PlacementResultPage'
import { ProfilePage } from './pages/ProfilePage'
import { VocabularyDetailsPage } from './pages/VocabularyDetailsPage'
import { VocabularyPage } from './pages/VocabularyPage'
import { AchievementsPage } from './pages/AchievementsPage'
import { ReviewPage } from './pages/ReviewPage'
import { ReviewSessionPage } from './pages/ReviewSessionPage'
import { PronunciationPage } from './pages/PronunciationPage'
import { DiagnosticsPage } from './pages/DiagnosticsPage'
import { NotFoundPage } from './pages/NotFoundPage'
import { GuidedLessonsPage } from './pages/GuidedLessonsPage'
import { GuidedLessonDetailPage } from './pages/GuidedLessonDetailPage'
import { GuidedLessonSessionPage } from './pages/GuidedLessonSessionPage'
import { CoursePage } from './pages/CoursePage'
import { PracticePage } from './pages/PracticePage'
import { PracticeSessionPage } from './pages/PracticeSessionPage'

function App() {
  return <Routes>
    <Route element={<AppLayout />}>
      <Route index element={<DashboardPage />} />
      <Route path="dashboard" element={<Navigate to="/" replace />} />
      <Route path="history" element={<HistoryPage />} />
      <Route path="history/:lessonId" element={<LessonDetailsPage />} />
      <Route path="lesson/new" element={<NewLessonPage />} />
      <Route path="guided-lessons" element={<GuidedLessonsPage />} />
      <Route path="guided-lessons/session/:sessionId" element={<GuidedLessonSessionPage />} />
      <Route path="guided-lessons/:lessonId" element={<GuidedLessonDetailPage />} />
      <Route path="course" element={<CoursePage />} />
      <Route path="course/:curriculumId" element={<CoursePage />} />
      <Route path="course/:curriculumId/:levelId" element={<CoursePage />} />
      <Route path="course/:curriculumId/:levelId/:unitId" element={<CoursePage />} />
      <Route path="progress" element={<ProgressPage />} />
      <Route path="achievements" element={<AchievementsPage />} />
      <Route path="review" element={<ReviewPage />} />
      <Route path="review/session/:sessionId" element={<ReviewSessionPage />} />
      <Route path="placement" element={<PlacementPage />} />
      <Route path="placement/results/:attemptId" element={<PlacementResultPage />} />
      <Route path="profile" element={<ProfilePage />} />
      <Route path="vocabulary" element={<VocabularyPage />} />
      <Route path="vocabulary/:vocabularyId" element={<VocabularyDetailsPage />} />
      <Route path="pronunciation" element={<PronunciationPage />} />
      <Route path="practice" element={<PracticePage />} />
      <Route path="practice/session/:sessionId" element={<PracticeSessionPage />} />
      <Route path="settings" element={<SettingsPage />} />
      <Route path="diagnostics" element={<DiagnosticsPage />} />
      <Route path="*" element={<NotFoundPage />} />
    </Route>
  </Routes>
}

export default App
