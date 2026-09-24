import '../../../core/network/api_client.dart';
import 'activity_entry.dart';

/// Wraps /activity-log - see backend/src/Modules/Activity/ActivityController.php.
/// Dashboard-only for now (the recent-activity panel); a full
/// paginated/searchable activity feature is future work.
class ActivityRepository {
  ActivityRepository({required ApiClient apiClient}) : _apiClient = apiClient;

  final ApiClient _apiClient;

  /// Most recent entries at the endpoint's own default page size -
  /// unpaginated, same as the web frontend's fetchActivity() (backs a
  /// notifications-style list, not a paginated log view).
  Future<List<ActivityEntry>> fetchRecent() async {
    final body = await _apiClient.get('/activity-log');
    return (body['data'] as List<dynamic>)
        .map((json) => ActivityEntry.fromJson(json as Map<String, dynamic>))
        .toList();
  }
}
