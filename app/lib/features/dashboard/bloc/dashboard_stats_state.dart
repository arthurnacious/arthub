part of 'dashboard_stats_bloc.dart';

enum DashboardStatsStatus { loading, success, failure }

class DashboardStatsState extends Equatable {
  const DashboardStatsState({
    this.status = DashboardStatsStatus.loading,
    this.clientsCount = 0,
    this.projectsCount = 0,
    this.errorMessage,
  });

  final DashboardStatsStatus status;
  final int clientsCount;
  final int projectsCount;
  final String? errorMessage;

  DashboardStatsState copyWith({
    DashboardStatsStatus? status,
    int? clientsCount,
    int? projectsCount,
    String? errorMessage,
  }) {
    return DashboardStatsState(
      status: status ?? this.status,
      clientsCount: clientsCount ?? this.clientsCount,
      projectsCount: projectsCount ?? this.projectsCount,
      errorMessage: errorMessage,
    );
  }

  @override
  List<Object?> get props => [status, clientsCount, projectsCount, errorMessage];
}
