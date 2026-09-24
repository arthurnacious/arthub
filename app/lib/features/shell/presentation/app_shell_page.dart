import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/coming_soon_page.dart';
import '../../../core/widgets/logo.dart';
import '../../auth/bloc/auth_bloc.dart';
import '../../clients/presentation/clients_page.dart';

class _NavEntry {
  const _NavEntry(this.label, this.icon, this.builder);

  final String label;
  final IconData icon;
  final WidgetBuilder builder;
}

final _navEntries = <_NavEntry>[
  _NavEntry('dashboard', Icons.dashboard_outlined, (_) => const ComingSoonPage(section: 'dashboard')),
  _NavEntry('clients', Icons.groups_outlined, (_) => const ClientsPage()),
  _NavEntry('projects', Icons.work_outline, (_) => const ComingSoonPage(section: 'projects')),
  _NavEntry('quotes', Icons.description_outlined, (_) => const ComingSoonPage(section: 'quotes')),
  _NavEntry('invoices', Icons.receipt_long_outlined, (_) => const ComingSoonPage(section: 'invoices')),
  _NavEntry('activity', Icons.timeline_outlined, (_) => const ComingSoonPage(section: 'activity')),
];

/// Same route list as the web frontend's ROUTES (frontend/src/lib/routes.ts),
/// as a Drawer instead of the web's always-visible sidebar, since that's
/// the idiomatic mobile equivalent. Only "clients" is wired to real data -
/// see ComingSoonPage for the rest.
class AppShellPage extends StatefulWidget {
  const AppShellPage({super.key});

  @override
  State<AppShellPage> createState() => _AppShellPageState();
}

class _AppShellPageState extends State<AppShellPage> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final entry = _navEntries[_selectedIndex];

    return Scaffold(
      appBar: AppBar(title: Text(entry.label)),
      drawer: Drawer(
        child: SafeArea(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Padding(
                padding: EdgeInsets.all(16),
                child: Logo(),
              ),
              const Divider(height: 1),
              Expanded(
                child: ListView(
                  padding: EdgeInsets.zero,
                  children: [
                    for (var i = 0; i < _navEntries.length; i++)
                      ListTile(
                        leading: Icon(
                          _navEntries[i].icon,
                          color: i == _selectedIndex ? AppColors.accent : AppColors.textDim,
                        ),
                        title: Text(
                          _navEntries[i].label,
                          style: TextStyle(
                            color: i == _selectedIndex ? AppColors.accent : AppColors.text,
                          ),
                        ),
                        selected: i == _selectedIndex,
                        selectedTileColor: AppColors.accentDim,
                        onTap: () {
                          setState(() => _selectedIndex = i);
                          Navigator.of(context).pop();
                        },
                      ),
                  ],
                ),
              ),
              const Divider(height: 1),
              ListTile(
                leading: const Icon(Icons.logout, color: AppColors.textDim),
                title: const Text('log out'),
                onTap: () => context.read<AuthBloc>().add(const AuthLogoutRequested()),
              ),
            ],
          ),
        ),
      ),
      body: entry.builder(context),
    );
  }
}
