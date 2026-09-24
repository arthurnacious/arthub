import 'package:arthub/core/network/api_client.dart';
import 'package:arthub/core/storage/token_storage.dart';
import 'package:arthub/features/activity/data/activity_entry.dart';
import 'package:arthub/features/activity/data/activity_repository.dart';
import 'package:arthub/features/clients/data/clients_repository.dart';
import 'package:arthub/features/invoices/data/invoice.dart';
import 'package:arthub/features/invoices/data/invoices_repository.dart';
import 'package:arthub/features/projects/data/projects_repository.dart';
import 'package:arthub/features/shell/presentation/app_shell_page.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';

// Never actually used (every method these fakes' base classes would call
// through it is overridden below), but ApiClient's constructor needs one.
final _dummyApiClient = ApiClient(tokenStorage: TokenStorage());

class _FakeClientsRepository extends ClientsRepository {
  _FakeClientsRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<ClientsPage> fetchClients({required int page, int pageSize = 20, String search = ''}) async =>
      const ClientsPage(data: [], page: 1, pageSize: 1, total: 3);
}

class _FakeProjectsRepository extends ProjectsRepository {
  _FakeProjectsRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<int> fetchCount() async => 5;
}

class _FakeInvoicesRepository extends InvoicesRepository {
  _FakeInvoicesRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<List<Invoice>> fetchAllForSummary() async => const [];
}

class _FakeActivityRepository extends ActivityRepository {
  _FakeActivityRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<List<ActivityEntry>> fetchRecent() async => const [];
}

Widget _wrap(Widget child) {
  return MultiRepositoryProvider(
    providers: [
      RepositoryProvider<ClientsRepository>(create: (_) => _FakeClientsRepository()),
      RepositoryProvider<ProjectsRepository>(create: (_) => _FakeProjectsRepository()),
      RepositoryProvider<InvoicesRepository>(create: (_) => _FakeInvoicesRepository()),
      RepositoryProvider<ActivityRepository>(create: (_) => _FakeActivityRepository()),
    ],
    child: MaterialApp(home: child),
  );
}

void main() {
  testWidgets('shows a bottom NavigationBar with one destination per section, no Drawer', (tester) async {
    await tester.pumpWidget(_wrap(const AppShellPage()));
    await tester.pump();

    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.byType(Drawer), findsNothing);
    for (final label in ['dashboard', 'clients', 'projects', 'quotes', 'invoices']) {
      expect(find.text(label), findsWidgets);
    }
  });

  testWidgets('defaults to the dashboard tab', (tester) async {
    await tester.pumpWidget(_wrap(const AppShellPage()));
    await tester.pump();

    expect(find.text('status'), findsOneWidget);
    expect(find.text('invoice_status'), findsOneWidget);
    expect(find.text('recent_activity'), findsOneWidget);
  });
}
