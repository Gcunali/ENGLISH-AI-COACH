import { Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import { DashboardPage } from './pages/DashboardPage'
import { HistoryPage } from './pages/HistoryPage'
import { LessonDetailsPage } from './pages/LessonDetailsPage'
import { PlaceholderPage } from './pages/PlaceholderPage'
import { ProgressPage } from './pages/ProgressPage'

function App() {
  return <Routes>
    <Route element={<AppLayout />}>
      <Route index element={<DashboardPage />} />
      <Route path="dashboard" element={<Navigate to="/" replace />} />
      <Route path="history" element={<HistoryPage />} />
      <Route path="history/:lessonId" element={<LessonDetailsPage />} />
      <Route path="progress" element={<ProgressPage />} />
      <Route path="vocabulary" element={<PlaceholderPage title="Vocabulary" />} />
      <Route path="settings" element={<PlaceholderPage title="Settings" />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Route>
  </Routes>
}

export default App
