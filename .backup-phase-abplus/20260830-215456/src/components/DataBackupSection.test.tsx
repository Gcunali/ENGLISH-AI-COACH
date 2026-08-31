// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
const native=vi.hoisted(()=>({getBackupStatus:vi.fn(),listAppBackups:vi.fn(),createAppBackup:vi.fn(),restoreAppBackup:vi.fn(),openBackupFolder:vi.fn()}))
vi.mock('../services/native',()=>native)
import { DataBackupSection } from './DataBackupSection'
const backup={backupId:'EnglishAICoach-Backup-1.eacbackup',createdAt:'2026-08-23T00:00:00Z',path:'C:\\backups\\one',databaseBytes:4096,databaseSha256:'A'.repeat(64),schemaVersion:13,valid:true}
beforeEach(()=>{native.getBackupStatus.mockResolvedValue({operation:'idle',error:null,backupDirectory:'C:\\backups',lastBackup:backup,restoreAllowed:true,restoreBlockReason:null,pendingRestart:false,lastRestore:null});native.listAppBackups.mockResolvedValue([backup]);native.createAppBackup.mockResolvedValue(backup);native.restoreAppBackup.mockResolvedValue({backupId:backup.backupId,safetyBackupId:'safety.eacbackup',restartRequired:true,message:'Restart the app to finish restore.'});native.openBackupFolder.mockResolvedValue(undefined)})
afterEach(()=>{cleanup();vi.clearAllMocks()})
describe('DataBackupSection',()=>{
  it('creates a backup and shows the last validated backup',async()=>{render(<MemoryRouter><DataBackupSection/></MemoryRouter>);expect(await screen.findByText(/Last backup/)).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Create Backup'}));await waitFor(()=>expect(native.createAppBackup).toHaveBeenCalledTimes(1))})
  it('requires explicit confirmation before scheduling restore',async()=>{render(<MemoryRouter><DataBackupSection/></MemoryRouter>);const restore=await screen.findByRole('button',{name:/Restore backup/});fireEvent.click(restore);expect(screen.getByRole('dialog')).toHaveTextContent('A safety backup of the current database will be created first.');expect(native.restoreAppBackup).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Restore Backup'}));await waitFor(()=>expect(native.restoreAppBackup).toHaveBeenCalledWith(backup.backupId))})
  it('disables restore while runtime protection blocks it',async()=>{native.getBackupStatus.mockResolvedValue({...await native.getBackupStatus(),restoreAllowed:false,restoreBlockReason:'End the active Voice Lesson before restoring data.'});render(<MemoryRouter><DataBackupSection/></MemoryRouter>);expect(await screen.findByRole('button',{name:/Restore backup/})).toBeDisabled();expect(screen.getByText(/active Voice Lesson/)).toBeInTheDocument()})
  it('shows failed backup operations accessibly',async()=>{native.createAppBackup.mockRejectedValue(new Error('backup failed safely'));render(<MemoryRouter><DataBackupSection/></MemoryRouter>);await screen.findByText(/Last backup/);fireEvent.click(screen.getByRole('button',{name:'Create Backup'}));expect(await screen.findByRole('alert')).toHaveTextContent('backup failed safely')})
})
