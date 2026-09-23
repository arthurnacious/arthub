import { useId, useState } from 'react'

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'

// Uncontrolled-from-the-parent - the parent doesn't need this form's
// values, only the final { currentPassword, newPassword } on submit. It
// keeps its own local state, validated client-side (mirrors the
// backend's own 8-character minimum and the current/new password check)
// before ever calling onSubmit. The parent forces a reset after a
// successful change by remounting this with a new `key`.
export function ChangePasswordForm({
  onSubmit,
  submitting,
}: {
  onSubmit: (values: { currentPassword: string; newPassword: string }) => void
  submitting: boolean
}) {
  const idPrefix = useId()
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')

  const tooShort = newPassword !== '' && newPassword.length < 8
  const mismatch = confirmPassword !== '' && newPassword !== confirmPassword
  const canSubmit = currentPassword !== '' && newPassword !== '' && !tooShort && !mismatch

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault()
        if (!canSubmit) return
        onSubmit({ currentPassword, newPassword })
      }}
      className="flex flex-col gap-3"
    >
      <label
        htmlFor={`${idPrefix}-current`}
        className="flex flex-col gap-1 text-xs text-[var(--text-dim)]"
      >
        current password
        <input
          id={`${idPrefix}-current`}
          type="password"
          value={currentPassword}
          onChange={(event) => setCurrentPassword(event.target.value)}
          className={FIELD_CLASSES}
        />
      </label>
      <label htmlFor={`${idPrefix}-new`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        new password
        <input
          id={`${idPrefix}-new`}
          type="password"
          value={newPassword}
          onChange={(event) => setNewPassword(event.target.value)}
          className={FIELD_CLASSES}
        />
      </label>
      <label
        htmlFor={`${idPrefix}-confirm`}
        className="flex flex-col gap-1 text-xs text-[var(--text-dim)]"
      >
        confirm new password
        <input
          id={`${idPrefix}-confirm`}
          type="password"
          value={confirmPassword}
          onChange={(event) => setConfirmPassword(event.target.value)}
          className={FIELD_CLASSES}
        />
      </label>
      {tooShort && <p className="text-xs text-red-400">must be at least 8 characters</p>}
      {mismatch && <p className="text-xs text-red-400">passwords don't match</p>}
      <div className="mt-2">
        <button
          type="submit"
          disabled={submitting || !canSubmit}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors disabled:opacity-50"
        >
          {submitting ? 'saving...' : 'change password'}
        </button>
      </div>
    </form>
  )
}
