import 'package:arthub/core/widgets/status_badge.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('renders its label', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Material(child: StatusBadge(label: 'active', tone: StatusTone.success)),
      ),
    );

    expect(find.text('active'), findsOneWidget);
  });
}
