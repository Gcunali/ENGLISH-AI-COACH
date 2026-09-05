import { Navigate, Route, Routes } from "react-router-dom";
import { AppLayout } from "./components/AppLayout";
import { DashboardPage } from "./pages/DashboardPage";
import { HistoryPage } from "./pages/HistoryPage";
import { LessonDetailsPage } from "./pages/LessonDetailsPage";
import { NewLessonPage } from "./pages/NewLessonPage";
import { SettingsPage } from "./pages/SettingsPage";
import { ProgressPage } from "./pages/ProgressPage";
import { PlacementPage } from "./pages/PlacementPage";
import { PlacementResultPage } from "./pages/PlacementResultPage";
import { ProfilePage } from "./pages/ProfilePage";
import { VocabularyDetailsPage } from "./pages/VocabularyDetailsPage";
import { VocabularyPage } from "./pages/VocabularyPage";
import { AchievementsPage } from "./pages/AchievementsPage";
import { ReviewPage } from "./pages/ReviewPage";
import { ReviewSessionPage } from "./pages/ReviewSessionPage";
import { PronunciationPage } from "./pages/PronunciationPage";
import { DiagnosticsPage } from "./pages/DiagnosticsPage";
import { NotFoundPage } from "./pages/NotFoundPage";
import { GuidedLessonsPage } from "./pages/GuidedLessonsPage";
import { GuidedLessonDetailPage } from "./pages/GuidedLessonDetailPage";
import { GuidedLessonSessionPage } from "./pages/GuidedLessonSessionPage";
import { CoursePage } from "./pages/CoursePage";
import { PracticePage } from "./pages/PracticePage";
import { PracticeSessionPage } from "./pages/PracticeSessionPage";
import { ToeicPage } from "./pages/ToeicPage";
import { ToeicSessionPage } from "./pages/ToeicSessionPage";
import { ToeicResultsPage } from "./pages/ToeicResultsPage";
import { ToeicHistoryPage } from "./pages/ToeicHistoryPage";
import { ToeicPart2SessionPage } from "./pages/ToeicPart2SessionPage";
import { ToeicPart2ResultsPage } from "./pages/ToeicPart2ResultsPage";
import { ToeicPart3SessionPage } from "./pages/ToeicPart3SessionPage";
import { ToeicPart3ResultsPage } from "./pages/ToeicPart3ResultsPage";
import { ToeicPart4SessionPage } from "./pages/ToeicPart4SessionPage";
import { ToeicPart4ResultsPage } from "./pages/ToeicPart4ResultsPage";
import { ToeicFullListeningPage } from "./pages/ToeicFullListeningPage";
import { ToeicPart5SessionPage } from "./pages/ToeicPart5SessionPage";
import { ToeicPart5ResultsPage } from "./pages/ToeicPart5ResultsPage";
import { ToeicPart6SessionPage } from "./pages/ToeicPart6SessionPage";
import { ToeicPart6ResultsPage } from "./pages/ToeicPart6ResultsPage";
import { ToeicPart7SessionPage } from "./pages/ToeicPart7SessionPage";
import { ToeicPart7ResultsPage } from "./pages/ToeicPart7ResultsPage";
import { ToeicFullReadingPage } from "./pages/ToeicFullReadingPage";
import { ToeicFullLrPage } from "./pages/ToeicFullLrPage";
import { ToeicPersonalizedPage } from "./pages/ToeicPersonalizedPage";

function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<DashboardPage />} />
        <Route path="dashboard" element={<Navigate to="/" replace />} />
        <Route path="history" element={<HistoryPage />} />
        <Route path="history/:lessonId" element={<LessonDetailsPage />} />
        <Route path="lesson/new" element={<NewLessonPage />} />
        <Route path="guided-lessons" element={<GuidedLessonsPage />} />
        <Route
          path="guided-lessons/session/:sessionId"
          element={<GuidedLessonSessionPage />}
        />
        <Route
          path="guided-lessons/:lessonId"
          element={<GuidedLessonDetailPage />}
        />
        <Route path="course" element={<CoursePage />} />
        <Route path="course/:curriculumId" element={<CoursePage />} />
        <Route path="course/:curriculumId/:levelId" element={<CoursePage />} />
        <Route
          path="course/:curriculumId/:levelId/:unitId"
          element={<CoursePage />}
        />
        <Route path="progress" element={<ProgressPage />} />
        <Route path="achievements" element={<AchievementsPage />} />
        <Route path="review" element={<ReviewPage />} />
        <Route
          path="review/session/:sessionId"
          element={<ReviewSessionPage />}
        />
        <Route path="placement" element={<PlacementPage />} />
        <Route
          path="placement/results/:attemptId"
          element={<PlacementResultPage />}
        />
        <Route path="profile" element={<ProfilePage />} />
        <Route path="vocabulary" element={<VocabularyPage />} />
        <Route
          path="vocabulary/:vocabularyId"
          element={<VocabularyDetailsPage />}
        />
        <Route path="pronunciation" element={<PronunciationPage />} />
        <Route path="practice" element={<PracticePage />} />
        <Route
          path="practice/session/:sessionId"
          element={<PracticeSessionPage />}
        />
        <Route path="toeic" element={<ToeicPage />} />
        <Route path="toeic/session/:sessionId" element={<ToeicSessionPage />} />
        <Route path="toeic/results/:sessionId" element={<ToeicResultsPage />} />
        <Route
          path="toeic/part2/session/:sessionId"
          element={<ToeicPart2SessionPage />}
        />
        <Route
          path="toeic/part2/results/:sessionId"
          element={<ToeicPart2ResultsPage />}
        />
        <Route
          path="toeic/part3/session/:sessionId"
          element={<ToeicPart3SessionPage />}
        />
        <Route
          path="toeic/part3/results/:sessionId"
          element={<ToeicPart3ResultsPage />}
        />
        <Route
          path="toeic/part4/session/:sessionId"
          element={<ToeicPart4SessionPage />}
        />
        <Route
          path="toeic/part4/results/:sessionId"
          element={<ToeicPart4ResultsPage />}
        />
        <Route path="toeic/listening" element={<ToeicFullListeningPage />} />
        <Route
          path="toeic/listening/:id"
          element={<ToeicFullListeningPage />}
        />
        <Route
          path="toeic/part5/session/:sessionId"
          element={<ToeicPart5SessionPage />}
        />
        <Route
          path="toeic/part5/results/:sessionId"
          element={<ToeicPart5ResultsPage />}
        />
        <Route
          path="toeic/part6/session/:sessionId"
          element={<ToeicPart6SessionPage />}
        />
        <Route
          path="toeic/part6/results/:sessionId"
          element={<ToeicPart6ResultsPage />}
        />
        <Route
          path="toeic/part7/session/:sessionId"
          element={<ToeicPart7SessionPage />}
        />
        <Route
          path="toeic/part7/results/:sessionId"
          element={<ToeicPart7ResultsPage />}
        />
        <Route path="toeic/reading" element={<ToeicFullReadingPage />} />
        <Route path="toeic/reading/:id" element={<ToeicFullReadingPage />} />
        <Route path="toeic/full" element={<ToeicFullLrPage />} />
        <Route path="toeic/full/:id" element={<ToeicFullLrPage />} />
        <Route path="toeic/history" element={<ToeicHistoryPage />} />
        <Route path="toeic/personalized" element={<ToeicPersonalizedPage />} />
        <Route path="toeic/personalized/:id" element={<ToeicPersonalizedPage />} />
        <Route path="settings" element={<SettingsPage />} />
        <Route path="diagnostics" element={<DiagnosticsPage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}

export default App;
