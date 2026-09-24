import 'package:arthub/core/network/api_client.dart';
import 'package:arthub/core/network/api_exception.dart';
import 'package:arthub/core/storage/token_storage.dart';
import 'package:arthub/features/clients/data/clients_repository.dart';
import 'package:arthub/features/dashboard/bloc/dashboard_stats_bloc.dart';
import 'package:arthub/features/projects/data/projects_repository.dart';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';

// Never actually used (every method the fakes below need is overridden),
// but ApiClient's constructor needs one.
final _dummyApiClient = ApiClient(tokenStorage: TokenStorage());

class _FakeClientsRepository extends ClientsRepository {
  _FakeClientsRepository(this.total) : super(apiClient: _dummyApiClient);
  final int total;

  @override
  Future<ClientsPage> fetchClients({required int page, int pageSize = 20, String search = ''}) async =>
      ClientsPage(data: const [], page: 1, pageSize: 1, total: total);
}

class _FakeProjectsRepository extends ProjectsRepository {
  _FakeProjectsRepository(this.count) : super(apiClient: _dummyApiClient);
  final int count;

  @override
  Future<int> fetchCount() async => count;
}

class _FailingProjectsRepository extends ProjectsRepository {
  _FailingProjectsRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<int> fetchCount() async => throw const ApiException('boom', 500);
}

void main() {
  group('DashboardStatsBloc', () {
    blocTest<DashboardStatsBloc, DashboardStatsState>(
      'emits success with both counts',
      build: () => DashboardStatsBloc(
        clientsRepository: _FakeClientsRepository(3),
        projectsRepository: _FakeProjectsRepository(5),
      ),
      act: (bloc) => bloc.add(const DashboardStatsRequested()),
      expect: () => [
        const DashboardStatsState(status: DashboardStatsStatus.loading),
        const DashboardStatsState(status: DashboardStatsStatus.success, clientsCount: 3, projectsCount: 5),
      ],
    );

    blocTest<DashboardStatsBloc, DashboardStatsState>(
      'emits failure with the error message when a fetch throws',
      build: () => DashboardStatsBloc(
        clientsRepository: _FakeClientsRepository(3),
        projectsRepository: _FailingProjectsRepository(),
      ),
      act: (bloc) => bloc.add(const DashboardStatsRequested()),
      expect: () => [
        const DashboardStatsState(status: DashboardStatsStatus.loading),
        isA<DashboardStatsState>()
            .having((s) => s.status, 'status', DashboardStatsStatus.failure)
            .having((s) => s.errorMessage, 'errorMessage', 'boom'),
      ],
    );
  });
}
