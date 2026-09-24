import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/coming_soon_page.dart';
import '../../auth/bloc/auth_bloc.dart';
import '../../clients/presentation/clients_page.dart';

class _NavEntry {
  const _NavEntry(this.label, this.icon, this.builder);

  final String label;
  final IconData icon;
  final WidgetBuilder builder;
}

// Material's own guidance caps a bottom nav at 5 destinations - unlike
// the web frontend's sidebar (frontend/src/lib/routes.ts), which has no
// such limit and lists all 6 sections (dashboard/clients/projects/
// quotes/invoices/activity). Activity is the one dropped from here: it's
// a read-only log rather than a primary workflow destination, so it
// lives behind the AppBar's history icon instead (pushed, not a tab) -
// see _openActivity below.
final _navEntries = <_NavEntry>[
  _NavEntry('dashboard', Icons.dashboard_outlined, (_) => const ComingSoonPage(section: 'dashboard')),
  _NavEntry('clients', Icons.groups_outlined, (_) => const ClientsPage()),
  _NavEntry('projects', Icons.work_outline, (_) => const ComingSoonPage(section: 'projects')),
  _NavEntry('quotes', Icons.description_outlined, (_) => const ComingSoonPage(section: 'quotes')),
  _NavEntry('invoices', Icons.receipt_long_outlined, (_) => const ComingSoonPage(section: 'invoices')),
];

/// This app's shell is bottom-nav, not the web frontend's sidebar - same
/// look and feel (theme, panel chrome), but a different navigation shape
/// for a different medium: this is a real Android app, where bottom
/// navigation is the idiomatic pattern, not a sidebar meant for a wide
/// desktop viewport. See AppTheme's navigationBarTheme for the styling.
class AppShellPage extends StatefulWidget {
  const AppShellPage({super.key});

  @override
  State<AppShellPage> createState() => _AppShellPageState();
}

class _AppShellPageState extends State<AppShellPage> {
  int _selectedIndex = 0;

  void _openActivity() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => Scaffold(
          appBar: AppBar(title: const Text('activity')),
          body: const ComingSoonPage(section: 'activity'),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final entry = _navEntries[_selectedIndex];

    return Scaffold(
      appBar: AppBar(
        title: Text(entry.label),
        actions: [
          IconButton(
            icon: const Icon(Icons.history),
            tooltip: 'activity',
            onPressed: _openActivity,
          ),
          IconButton(
            icon: const Icon(Icons.logout),
            tooltip: 'log out',
            onPressed: () => context.read<AuthBloc>().add(const AuthLogoutRequested()),
          ),
        ],
      ),
      body: entry.builder(context),
      bottomNavigationBar: DecoratedBox(
        // A top border, not elevation/shadow - matches every other bit of
        // chrome in this theme (AppBar, Panel), which uses borders instead.
        decoration: const BoxDecoration(
          border: Border(top: BorderSide(color: AppColors.border)),
        ),
        child: NavigationBar(
          selectedIndex: _selectedIndex,
          onDestinationSelected: (index) => setState(() => _selectedIndex = index),
          destinations: [
            for (final navEntry in _navEntries)
              NavigationDestination(icon: Icon(navEntry.icon), label: navEntry.label),
          ],
        ),
      ),
    );
  }
}
