import 'package:arthub/features/shell/presentation/app_shell_page.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('shows a bottom NavigationBar with one destination per section, no Drawer', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: AppShellPage()));

    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.byType(Drawer), findsNothing);
    for (final label in ['dashboard', 'clients', 'projects', 'quotes', 'invoices']) {
      expect(find.text(label), findsWidgets);
    }
  });

  testWidgets('defaults to the dashboard tab', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: AppShellPage()));

    expect(find.text('-- not built out yet, see ClientsPage for the pattern to follow --'), findsOneWidget);
  });
}
