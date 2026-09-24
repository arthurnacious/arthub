part of 'clients_bloc.dart';

enum ClientsStatus { initial, loading, loadingMore, success, failure }

class ClientsState extends Equatable {
  const ClientsState({
    this.status = ClientsStatus.initial,
    this.clients = const [],
    this.page = 1,
    this.hasNextPage = true,
    this.search = '',
    this.errorMessage,
  });

  final ClientsStatus status;
  final List<Client> clients;
  final int page;
  final bool hasNextPage;
  final String search;
  final String? errorMessage;

  ClientsState copyWith({
    ClientsStatus? status,
    List<Client>? clients,
    int? page,
    bool? hasNextPage,
    String? search,
    String? errorMessage,
  }) {
    return ClientsState(
      status: status ?? this.status,
      clients: clients ?? this.clients,
      page: page ?? this.page,
      hasNextPage: hasNextPage ?? this.hasNextPage,
      search: search ?? this.search,
      errorMessage: errorMessage,
    );
  }

  @override
  List<Object?> get props => [status, clients, page, hasNextPage, search, errorMessage];
}
