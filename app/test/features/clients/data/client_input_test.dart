import 'package:arthub/features/clients/data/client.dart';
import 'package:arthub/features/clients/data/client_input.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ClientInput', () {
    test('.blank() starts empty with the default status', () {
      const input = ClientInput.blank();

      expect(input.company, '');
      expect(input.contact, '');
      expect(input.email, '');
      expect(input.status, 'active');
    });

    test('.fromClient() copies fields, turning null contact/email into empty strings', () {
      const client = Client(
        id: 'c1',
        company: 'Acme Co',
        contact: null,
        email: null,
        status: 'prospect',
        projects: 0,
      );

      final input = ClientInput.fromClient(client);

      expect(input.company, 'Acme Co');
      expect(input.contact, '');
      expect(input.email, '');
      expect(input.status, 'prospect');
    });

    test('toJson() matches CreateClientDto/UpdateClientDto field names', () {
      const input = ClientInput(
        company: 'Acme Co',
        contact: 'Jane Doe',
        email: 'jane@acme.test',
        status: 'active',
      );

      expect(input.toJson(), {
        'company': 'Acme Co',
        'contact': 'Jane Doe',
        'email': 'jane@acme.test',
        'status': 'active',
      });
    });

    test('copyWith() only replaces the given fields', () {
      const input = ClientInput.blank();

      final updated = input.copyWith(company: 'Acme Co', status: 'inactive');

      expect(updated.company, 'Acme Co');
      expect(updated.status, 'inactive');
      expect(updated.contact, input.contact);
      expect(updated.email, input.email);
    });
  });
}
