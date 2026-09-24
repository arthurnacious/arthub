part of 'recent_activity_bloc.dart';

sealed class RecentActivityEvent extends Equatable {
  const RecentActivityEvent();

  @override
  List<Object?> get props => [];
}

class RecentActivityRequested extends RecentActivityEvent {
  const RecentActivityRequested();
}
