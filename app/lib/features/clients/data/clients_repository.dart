import '../../../core/network/api_client.dart';
import 'client.dart';
import 'client_input.dart';

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
/// backend/src/Modules/Clients/ClientsController.php.
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

  Future<Client> fetchClient(String id) async {
    final body = await _apiClient.get('/clients/$id');
    return Client.fromJson(body);
  }

  Future<Client> createClient(ClientInput input) async {
    final body = await _apiClient.post('/clients', body: input.toJson());
    return Client.fromJson(body);
  }

  Future<Client> updateClient(String id, ClientInput input) async {
    final body = await _apiClient.patch('/clients/$id', body: input.toJson());
    return Client.fromJson(body);
  }

  Future<void> deleteClient(String id) => _apiClient.delete('/clients/$id');
}
