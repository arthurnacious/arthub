/// Short "2m ago" style relative time - ported from the web frontend's
/// formatRelativeTime (frontend/src/lib/format-relative-time.ts). `now`
/// is a parameter (not DateTime.now() internally) so it's actually
/// deterministic to test.
String formatRelativeTime(String iso, {DateTime? now}) {
  final reference = now ?? DateTime.now();
  final diffSec = reference.difference(DateTime.parse(iso)).inSeconds;

  if (diffSec < 60) return 'just now';

  final diffMin = (diffSec / 60).round();
  if (diffMin < 60) return '${diffMin}m ago';

  final diffHour = (diffMin / 60).round();
  if (diffHour < 24) return '${diffHour}h ago';

  final diffDay = (diffHour / 24).round();
  return '${diffDay}d ago';
}
