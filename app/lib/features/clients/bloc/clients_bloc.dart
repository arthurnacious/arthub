import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../data/client.dart';
import '../data/clients_repository.dart';

part 'clients_event.dart';
part 'clients_state.dart';

const _pageSize = 20;

/// Infinite-scroll list state for the Clients page - one query-driven
/// bloc per list-style feature is the pattern to copy for
/// Projects/Quotes/Invoices/Activity when they're built out (see
/// clients_page.dart for how it's consumed).
class ClientsBloc extends Bloc<ClientsEvent, ClientsState> {
  ClientsBloc({required ClientsRepository clientsRepository})
      : _clientsRepository = clientsRepository,
        super(const ClientsState()) {
    on<ClientsRequested>(_onRequested);
    on<ClientsSearchChanged>(_onSearchChanged);
    on<ClientsNextPageRequested>(_onNextPageRequested);
  }

  final ClientsRepository _clientsRepository;

  Future<void> _onRequested(ClientsRequested event, Emitter<ClientsState> emit) async {
    await _loadFirstPage(emit, search: state.search);
  }

  Future<void> _onSearchChanged(ClientsSearchChanged event, Emitter<ClientsState> emit) async {
    await _loadFirstPage(emit, search: event.query);
  }

  Future<void> _loadFirstPage(Emitter<ClientsState> emit, {required String search}) async {
    emit(state.copyWith(status: ClientsStatus.loading, search: search));

    try {
      final result = await _clientsRepository.fetchClients(page: 1, pageSize: _pageSize, search: search);
      emit(state.copyWith(
        status: ClientsStatus.success,
        clients: result.data,
        page: result.page,
        hasNextPage: result.hasNextPage,
      ));
    } on ApiException catch (error) {
      emit(state.copyWith(status: ClientsStatus.failure, errorMessage: error.message));
    }
  }

  Future<void> _onNextPageRequested(
    ClientsNextPageRequested event,
    Emitter<ClientsState> emit,
  ) async {
    if (!state.hasNextPage || state.status == ClientsStatus.loadingMore) return;

    emit(state.copyWith(status: ClientsStatus.loadingMore));

    try {
      final result = await _clientsRepository.fetchClients(
        page: state.page + 1,
        pageSize: _pageSize,
        search: state.search,
      );
      emit(state.copyWith(
        status: ClientsStatus.success,
        clients: [...state.clients, ...result.data],
        page: result.page,
        hasNextPage: result.hasNextPage,
      ));
    } on ApiException catch (error) {
      // Keep the already-loaded clients visible; only surface the error.
      emit(state.copyWith(status: ClientsStatus.success, errorMessage: error.message));
    }
  }
}
