part of 'dashboard_stats_bloc.dart';

sealed class DashboardStatsEvent extends Equatable {
  const DashboardStatsEvent();

  @override
  List<Object?> get props => [];
}

class DashboardStatsRequested extends DashboardStatsEvent {
  const DashboardStatsRequested();
}
