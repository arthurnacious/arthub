import 'package:arthub/features/invoices/data/invoice.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Invoice.fromJson', () {
    test('parses a paid invoice', () {
      final invoice = Invoice.fromJson({
        'id': 'i1',
        'status': 'paid',
        'total': 500.5,
        'paidAt': '2026-09-01 10:00:00+02',
      });

      expect(invoice.id, 'i1');
      expect(invoice.status, 'paid');
      expect(invoice.total, 500.5);
      expect(invoice.paidAt, '2026-09-01 10:00:00+02');
    });

    test('parses an unpaid invoice with a null paidAt and a whole-number total', () {
      final invoice = Invoice.fromJson({'id': 'i2', 'status': 'draft', 'total': 100, 'paidAt': null});

      expect(invoice.total, 100.0);
      expect(invoice.paidAt, isNull);
    });
  });
}
