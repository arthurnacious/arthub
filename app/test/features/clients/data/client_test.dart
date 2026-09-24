import 'package:arthub/features/clients/data/client.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Client.fromJson', () {
    test('parses a full ClientsController::present() row', () {
      final client = Client.fromJson({
        'id': 'c1',
        'company': 'Acme Co',
        'contact': 'Jane Doe',
        'email': 'jane@acme.test',
        'status': 'active',
        'projects': 3,
      });

      expect(client.id, 'c1');
      expect(client.company, 'Acme Co');
      expect(client.contact, 'Jane Doe');
      expect(client.email, 'jane@acme.test');
      expect(client.status, 'active');
      expect(client.projects, 3);
    });

    test('tolerates null contact/email, same as the backend allows', () {
      final client = Client.fromJson({
        'id': 'c1',
        'company': 'Acme Co',
        'contact': null,
        'email': null,
        'status': 'prospect',
        'projects': 0,
      });

      expect(client.contact, isNull);
      expect(client.email, isNull);
    });
  });
}
