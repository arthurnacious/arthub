import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../data/client.dart';
import '../data/client_input.dart';
import '../data/clients_repository.dart';

part 'client_detail_event.dart';
part 'client_detail_state.dart';

/// One instance per client detail screen (created in the route, not
/// app-wide like AuthBloc/ClientsBloc) - see
/// clients_page.dart/client_detail_page.dart. Owns viewing, editing
/// (toggled in place, same UX as the web's clients.$clientId.tsx) and
/// deleting a single client.
class ClientDetailBloc extends Bloc<ClientDetailEvent, ClientDetailState> {
  ClientDetailBloc({required ClientsRepository clientsRepository, required String clientId})
      : _clientsRepository = clientsRepository,
        _clientId = clientId,
        super(const ClientDetailState()) {
    on<ClientDetailRequested>(_onRequested);
    on<ClientDetailEditToggled>(_onEditToggled);
    on<ClientDetailFormChanged>(_onFormChanged);
    on<ClientDetailSubmitted>(_onSubmitted);
    on<ClientDetailDeleteRequested>(_onDeleteRequested);
  }

  final ClientsRepository _clientsRepository;
  final String _clientId;

  Future<void> _onRequested(ClientDetailRequested event, Emitter<ClientDetailState> emit) async {
    emit(state.copyWith(status: ClientDetailStatus.loading));

    try {
      final client = await _clientsRepository.fetchClient(_clientId);
      emit(state.copyWith(
        status: ClientDetailStatus.loaded,
        client: client,
        form: ClientInput.fromClient(client),
      ));
    } on ApiException catch (error) {
      emit(state.copyWith(status: ClientDetailStatus.failure, errorMessage: error.message));
    }
  }

  void _onEditToggled(ClientDetailEditToggled event, Emitter<ClientDetailState> emit) {
    emit(state.copyWith(
      editing: event.editing,
      // Reset any in-progress edits when re-entering edit mode.
      form: state.client != null ? ClientInput.fromClient(state.client!) : state.form,
    ));
  }

  void _onFormChanged(ClientDetailFormChanged event, Emitter<ClientDetailState> emit) {
    emit(state.copyWith(form: event.form));
  }

  Future<void> _onSubmitted(ClientDetailSubmitted event, Emitter<ClientDetailState> emit) async {
    emit(state.copyWith(status: ClientDetailStatus.saving));

    try {
      final updated = await _clientsRepository.updateClient(_clientId, state.form);
      emit(state.copyWith(status: ClientDetailStatus.loaded, client: updated, editing: false));
    } on ApiException catch (error) {
      emit(state.copyWith(status: ClientDetailStatus.loaded, errorMessage: error.message));
    }
  }

  Future<void> _onDeleteRequested(
    ClientDetailDeleteRequested event,
    Emitter<ClientDetailState> emit,
  ) async {
    emit(state.copyWith(status: ClientDetailStatus.deleting));

    try {
      await _clientsRepository.deleteClient(_clientId);
      emit(state.copyWith(status: ClientDetailStatus.deleted));
    } on ApiException catch (error) {
      emit(state.copyWith(status: ClientDetailStatus.loaded, errorMessage: error.message));
    }
  }
}
