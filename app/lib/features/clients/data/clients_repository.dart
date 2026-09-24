import '../../../core/network/api_client.dart';
import 'client.dart';

/// One page of a paginated list endpoint - every list endpoint in this
/// API shares this envelope (see e.g. ClientsController::index()).
class ClientsPage {
  const ClientsPage({
    required this.data,
    required this.page,
    required this.pageSize,
    required this.total,
  });

  final List<Client> data;
  final int page;
  final int pageSize;
  final int total;

  bool get hasNextPage => page * pageSize < total;
}

/// Wraps the /clients endpoints - see
/// backend/src/Modules/Clients/ClientsController.php. Only read (list)
/// is implemented for this scaffold; create/update/delete follow the
/// same request/response shape and are natural next additions once
/// there's a form UI to drive them (see frontend's ClientForm.tsx for
/// the field set: company, contact, email, status).
class ClientsRepository {
  ClientsRepository({required ApiClient apiClient}) : _apiClient = apiClient;

  final ApiClient _apiClient;

  Future<ClientsPage> fetchClients({
    required int page,
    int pageSize = 20,
    String search = '',
  }) async {
    final body = await _apiClient.get('/clients', query: {
      'page': page,
      'pageSize': pageSize,
      'q': search,
    });

    return ClientsPage(
      data: (body['data'] as List<dynamic>)
          .map((json) => Client.fromJson(json as Map<String, dynamic>))
          .toList(),
      page: body['page'] as int,
      pageSize: body['pageSize'] as int,
      total: body['total'] as int,
    );
  }
}
