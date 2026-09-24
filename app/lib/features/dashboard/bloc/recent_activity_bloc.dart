import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../../activity/data/activity_entry.dart';
import '../../activity/data/activity_repository.dart';

part 'recent_activity_event.dart';
part 'recent_activity_state.dart';

/// Backs the dashboard's "recent_activity" panel - matches the web
/// frontend's AuditLogPanel in spirit, but unpaginated/read-only (a
/// short recent list, not the full searchable log - see
/// ActivityRepository.fetchRecent()).
class RecentActivityBloc extends Bloc<RecentActivityEvent, RecentActivityState> {
  RecentActivityBloc({required ActivityRepository activityRepository})
      : _activityRepository = activityRepository,
        super(const RecentActivityState()) {
    on<RecentActivityRequested>(_onRequested);
  }

  final ActivityRepository _activityRepository;

  Future<void> _onRequested(
    RecentActivityRequested event,
    Emitter<RecentActivityState> emit,
  ) async {
    emit(state.copyWith(status: RecentActivityStatus.loading));

    try {
      final entries = await _activityRepository.fetchRecent();
      emit(state.copyWith(status: RecentActivityStatus.success, entries: entries));
    } on ApiException catch (error) {
      emit(state.copyWith(status: RecentActivityStatus.failure, errorMessage: error.message));
    }
  }
}
