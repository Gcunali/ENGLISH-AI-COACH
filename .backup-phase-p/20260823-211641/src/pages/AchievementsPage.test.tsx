// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ useAchievements: vi.fn(), useOverview: vi.fn(), getProfile: vi.fn(), updateGoal: vi.fn() }))
vi.mock('../hooks/useGamification', () => ({ useAchievements: mocks.useAchievements, useGamificationOverview: mocks.useOverview }))
vi.mock('../services/native', () => ({ getGamificationProfile: mocks.getProfile, updateWeeklyPracticeGoal: mocks.updateGoal }))
import { AchievementsPage } from './AchievementsPage'

const overview = { schemaVersion:1,totalXp:48,practiceLevel:1,currentLevelThreshold:0,nextLevelThreshold:100,xpIntoCurrentLevel:48,xpNeededForNextLevel:52,qualifyingLessonCount:1,totalPracticeMinutes:2,currentStreakDays:1,longestStreakDays:1,weeklyGoal:{goalMinutes:90,practicedMinutes:2,progressPercent:2,reached:false},unlockedAchievementCount:1,totalAchievementCount:11 }
const achievement = { id:'first_conversation',version:1,title:'First Conversation',description:'Complete your first real conversation lesson.',category:'practice',unlocked:true,unlockedAt:'2026-08-21T12:00:00Z',progressCurrent:1,progressTarget:1 }

describe('AchievementsPage', () => {
  beforeEach(() => { vi.clearAllMocks(); mocks.useOverview.mockReturnValue({data:overview,loading:false,error:null,reload:vi.fn()}); mocks.useAchievements.mockReturnValue({data:[achievement],loading:false,error:null,reload:vi.fn()}); mocks.getProfile.mockResolvedValue({schemaVersion:1,weeklyGoalMinutes:90}); mocks.updateGoal.mockResolvedValue({schemaVersion:1,weeklyGoalMinutes:105}) })
  afterEach(cleanup)
  it('renders factual practice progress and unlocked metadata', async () => { render(<MemoryRouter><AchievementsPage /></MemoryRouter>); expect(screen.getByRole('heading',{name:'Achievements'})).toBeInTheDocument(); expect(screen.getByText('First Conversation')).toBeInTheDocument(); expect(screen.getByText(/Practice Level and XP measure consistency/)).toBeInTheDocument(); await waitFor(()=>expect(mocks.getProfile).toHaveBeenCalled()) })
  it('persists a valid weekly goal', async () => { render(<MemoryRouter><AchievementsPage /></MemoryRouter>); const input=screen.getByLabelText('Weekly goal minutes'); fireEvent.change(input,{target:{value:'105'}}); fireEvent.click(screen.getByRole('button',{name:'Save goal'})); await waitFor(()=>expect(mocks.updateGoal).toHaveBeenCalledWith(105)); expect(await screen.findByText('Weekly goal saved locally.')).toBeInTheDocument() })
})
