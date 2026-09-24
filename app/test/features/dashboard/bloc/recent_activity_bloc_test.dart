import 'package:arthub/core/network/api_client.dart';
import 'package:arthub/core/network/api_exception.dart';
import 'package:arthub/core/storage/token_storage.dart';
import 'package:arthub/features/activity/data/activity_entry.dart';
import 'package:arthub/features/activity/data/activity_repository.dart';
import 'package:arthub/features/dashboard/bloc/recent_activity_bloc.dart';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';

final _dummyApiClient = ApiClient(tokenStorage: TokenStorage());

const _entry = ActivityEntry(
  id: 'a1',
  action: 'client.created',
  description: 'Created client Acme Co',
  subjectType: 'client',
  subjectId: 'c1',
  createdAt: '2026-09-24T10:00:00Z',
);

class _FakeActivityRepository extends ActivityRepository {
  _FakeActivityRepository(this.entries) : super(apiClient: _dummyApiClient);
  final List<ActivityEntry> entries;

  @override
  Future<List<ActivityEntry>> fetchRecent() async => entries;
}

class _FailingActivityRepository extends ActivityRepository {
  _FailingActivityRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<List<ActivityEntry>> fetchRecent() async => throw const ApiException('boom', 500);
}

void main() {
  group('RecentActivityBloc', () {
    blocTest<RecentActivityBloc, RecentActivityState>(
      'emits success with the fetched entries',
      build: () => RecentActivityBloc(activityRepository: _FakeActivityRepository(const [_entry])),
      act: (bloc) => bloc.add(const RecentActivityRequested()),
      expect: () => [
        const RecentActivityState(status: RecentActivityStatus.loading),
        const RecentActivityState(status: RecentActivityStatus.success, entries: [_entry]),
      ],
    );

    blocTest<RecentActivityBloc, RecentActivityState>(
      'emits failure with the error message when the fetch throws',
      build: () => RecentActivityBloc(activityRepository: _FailingActivityRepository()),
      act: (bloc) => bloc.add(const RecentActivityRequested()),
      expect: () => [
        const RecentActivityState(status: RecentActivityStatus.loading),
        isA<RecentActivityState>()
            .having((s) => s.status, 'status', RecentActivityStatus.failure)
            .having((s) => s.errorMessage, 'errorMessage', 'boom'),
      ],
    );
  });
}
