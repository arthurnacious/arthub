import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../../clients/data/clients_repository.dart';
import '../../projects/data/projects_repository.dart';

part 'dashboard_stats_event.dart';
part 'dashboard_stats_state.dart';

/// Backs the dashboard's "status" panel - matches the web frontend's
/// ClientsStatCard/ProjectsStatCard, combined into one bloc since
/// they're two trivially cheap counts rather than two separate network
/// round trips worth isolating.
class DashboardStatsBloc extends Bloc<DashboardStatsEvent, DashboardStatsState> {
  DashboardStatsBloc({
    required ClientsRepository clientsRepository,
    required ProjectsRepository projectsRepository,
  })  : _clientsRepository = clientsRepository,
        _projectsRepository = projectsRepository,
        super(const DashboardStatsState()) {
    on<DashboardStatsRequested>(_onRequested);
  }

  final ClientsRepository _clientsRepository;
  final ProjectsRepository _projectsRepository;

  Future<void> _onRequested(
    DashboardStatsRequested event,
    Emitter<DashboardStatsState> emit,
  ) async {
    emit(state.copyWith(status: DashboardStatsStatus.loading));

    try {
      final results = await Future.wait([
        _clientsRepository.fetchClients(page: 1, pageSize: 1).then((page) => page.total),
        _projectsRepository.fetchCount(),
      ]);

      emit(state.copyWith(
        status: DashboardStatsStatus.success,
        clientsCount: results[0],
        projectsCount: results[1],
      ));
    } on ApiException catch (error) {
      emit(state.copyWith(status: DashboardStatsStatus.failure, errorMessage: error.message));
    }
  }
}
