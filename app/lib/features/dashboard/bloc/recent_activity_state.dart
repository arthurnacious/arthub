part of 'recent_activity_bloc.dart';

enum RecentActivityStatus { loading, success, failure }

class RecentActivityState extends Equatable {
  const RecentActivityState({
    this.status = RecentActivityStatus.loading,
    this.entries = const [],
    this.errorMessage,
  });

  final RecentActivityStatus status;
  final List<ActivityEntry> entries;
  final String? errorMessage;

  RecentActivityState copyWith({
    RecentActivityStatus? status,
    List<ActivityEntry>? entries,
    String? errorMessage,
  }) {
    return RecentActivityState(
      status: status ?? this.status,
      entries: entries ?? this.entries,
      errorMessage: errorMessage,
    );
  }

  @override
  List<Object?> get props => [status, entries, errorMessage];
}
