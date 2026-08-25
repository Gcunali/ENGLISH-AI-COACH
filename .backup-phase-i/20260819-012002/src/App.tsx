import { Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import { DashboardPage } from './pages/DashboardPage'
import { HistoryPage } from './pages/HistoryPage'
import { LessonDetailsPage } from './pages/LessonDetailsPage'
import { SettingsPage } from './pages/SettingsPage'
import { ProgressPage } from './pages/ProgressPage'
import { VocabularyDetailsPage } from './pages/VocabularyDetailsPage'
import { VocabularyPage } from './pages/VocabularyPage'

function App() {
  return <Routes>
    <Route element={<AppLayout />}>
      <Route index element={<DashboardPage />} />
      <Route path="dashboard" element={<Navigate to="/" replace />} />
      <Route path="history" element={<HistoryPage />} />
      <Route path="history/:lessonId" element={<LessonDetailsPage />} />
      <Route path="progress" element={<ProgressPage />} />
      <Route path="vocabulary" element={<VocabularyPage />} />
      <Route path="vocabulary/:vocabularyId" element={<VocabularyDetailsPage />} />
      <Route path="settings" element={<SettingsPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Route>
  </Routes>
}

export default App
