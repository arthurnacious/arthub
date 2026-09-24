import 'package:arthub/features/clients/data/client_input.dart';
import 'package:arthub/features/clients/presentation/client_form_fields.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('typing in the company field reports the updated ClientInput', (tester) async {
    ClientInput? latest;

    await tester.pumpWidget(
      MaterialApp(
        home: Material(
          child: ClientFormFields(
            value: const ClientInput.blank(),
            onChanged: (value) => latest = value,
          ),
        ),
      ),
    );

    await tester.enterText(find.byType(TextFormField).first, 'Acme Co');

    expect(latest?.company, 'Acme Co');
  });

  testWidgets('shows every status option in the dropdown', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Material(
          child: ClientFormFields(value: const ClientInput.blank(), onChanged: (_) {}),
        ),
      ),
    );

    await tester.tap(find.byType(DropdownButtonFormField<String>));
    await tester.pumpAndSettle();

    for (final status in clientStatusOptions) {
      expect(find.text(status), findsWidgets);
    }
  });
}
