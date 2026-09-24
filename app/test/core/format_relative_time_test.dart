import 'package:arthub/core/format_relative_time.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  final now = DateTime(2026, 9, 24, 12, 0, 0);

  group('formatRelativeTime', () {
    test('under a minute reads "just now"', () {
      final iso = now.subtract(const Duration(seconds: 30)).toIso8601String();
      expect(formatRelativeTime(iso, now: now), 'just now');
    });

    test('minutes read "Nm ago"', () {
      final iso = now.subtract(const Duration(minutes: 5)).toIso8601String();
      expect(formatRelativeTime(iso, now: now), '5m ago');
    });

    test('hours read "Nh ago"', () {
      final iso = now.subtract(const Duration(hours: 3)).toIso8601String();
      expect(formatRelativeTime(iso, now: now), '3h ago');
    });

    test('days read "Nd ago"', () {
      final iso = now.subtract(const Duration(days: 2)).toIso8601String();
      expect(formatRelativeTime(iso, now: now), '2d ago');
    });
  });
}
