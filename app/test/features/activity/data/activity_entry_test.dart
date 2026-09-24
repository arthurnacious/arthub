import 'package:arthub/features/activity/data/activity_entry.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ActivityEntry.fromJson', () {
    test('parses the raw snake_case row ActivityController::index() returns', () {
      final entry = ActivityEntry.fromJson({
        'id': 'a1',
        'action': 'client.created',
        'description': 'Created client Acme Co',
        'subject_type': 'client',
        'subject_id': 'c1',
        'created_at': '2026-09-24T10:00:00Z',
      });

      expect(entry.id, 'a1');
      expect(entry.action, 'client.created');
      expect(entry.subjectType, 'client');
      expect(entry.subjectId, 'c1');
    });
  });

  group('linksToClient', () {
    test('true only when subject_type is client and subject_id is present', () {
      const clientEntry = ActivityEntry(
        id: '1',
        action: 'client.created',
        description: '',
        subjectType: 'client',
        subjectId: 'c1',
        createdAt: '2026-09-24T10:00:00Z',
      );
      const authEntry = ActivityEntry(
        id: '2',
        action: 'auth.login',
        description: '',
        subjectType: null,
        subjectId: null,
        createdAt: '2026-09-24T10:00:00Z',
      );

      expect(clientEntry.linksToClient, isTrue);
      expect(authEntry.linksToClient, isFalse);
    });
  });
}
