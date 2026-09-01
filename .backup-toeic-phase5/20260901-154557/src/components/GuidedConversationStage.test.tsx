// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { GuidedActiveContent, GuidedLessonSession, VoiceEngineEvent } from '../types'
import { GuidedConversationStage } from './GuidedConversationStage'

const mocks={start:vi.fn(),stop:vi.fn(),finish:vi.fn(),cancel:vi.fn(),get:vi.fn(),handler:null as ((event:VoiceEngineEvent)=>void)|null}
vi.mock('../services/native',()=>({
 startGuidedConversation:(sessionId:string,stageId:string)=>mocks.start(sessionId,stageId),stopGuidedConversation:()=>mocks.stop(),finishGuidedConversation:(sessionId:string,stageId:string)=>mocks.finish(sessionId,stageId),cancelCurrentTeacherResponse:()=>mocks.cancel(),getGuidedLessonSession:()=>mocks.get(),subscribeVoiceEngineEvents:async(handler:(event:VoiceEngineEvent)=>void)=>{mocks.handler=handler;return()=>undefined},
}))

const content:Extract<GuidedActiveContent,{kind:'guided_conversation'}>={kind:'guided_conversation',scenario:'You are ordering at a café.',studentRole:'Customer',teacherRole:'Barista',goal:'Order politely.',targetVocabulary:['coffee'],targetExpressions:['please'],minimumStudentTurns:3,recommendedStudentTurns:5,maximumStudentTurns:7,started:false,studentTurnCount:0,assistantTurnCount:0,turns:[]}
const session={id:'session-u',activeStage:{stageId:'guided',content}} as GuidedLessonSession

describe('GuidedConversationStage',()=>{afterEach(cleanup);beforeEach(()=>{vi.clearAllMocks();mocks.handler=null;mocks.get.mockResolvedValue(session);mocks.start.mockResolvedValue(session);mocks.stop.mockResolvedValue({state:'stopped',processId:null})})
 it('renders bounded lesson data without autostart and explains deterministic finish',()=>{render(<GuidedConversationStage content={content} session={session} stageId="guided" update={vi.fn()} reportError={vi.fn()}/>);expect(screen.getByText('You are ordering at a café.')).toBeInTheDocument();expect(screen.getByText('Helpful language, not a required checklist.')).toBeInTheDocument();expect(screen.getByRole('button',{name:'Finish Conversation'})).toBeDisabled();expect(mocks.start).not.toHaveBeenCalled()})
 it('starts only after the explicit gesture and reconciles final turns without token aria announcements',async()=>{const update=vi.fn();render(<GuidedConversationStage content={content} session={session} stageId="guided" update={update} reportError={vi.fn()}/>);fireEvent.click(screen.getByRole('button',{name:'Start Conversation'}));await waitFor(()=>expect(mocks.start).toHaveBeenCalledWith('session-u','guided'));expect(update).toHaveBeenCalled();expect(screen.getAllByRole('status')).toHaveLength(1)})
})
