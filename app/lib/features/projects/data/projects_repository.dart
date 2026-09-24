import '../../../core/network/api_client.dart';

/// Wraps /projects - see backend/src/Modules/Projects/ProjectsController.php.
/// Dashboard-only for now (just the count the "status" panel needs);
/// a real Projects feature (list/detail/CRUD, following
/// lib/features/clients/'s shape) is future work.
class ProjectsRepository {
  ProjectsRepository({required ApiClient apiClient}) : _apiClient = apiClient;

  final ApiClient _apiClient;

  Future<int> fetchCount() async {
    final body = await _apiClient.get('/projects', query: {'page': 1, 'pageSize': 1});
    return body['total'] as int;
  }
}
