import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../data/client_input.dart';
import '../data/clients_repository.dart';

part 'client_create_event.dart';
part 'client_create_state.dart';

/// One instance per "new client" screen - see client_create_page.dart.
class ClientCreateBloc extends Bloc<ClientCreateEvent, ClientCreateState> {
  ClientCreateBloc({required ClientsRepository clientsRepository})
      : _clientsRepository = clientsRepository,
        super(const ClientCreateState()) {
    on<ClientCreateFormChanged>((event, emit) => emit(state.copyWith(form: event.form)));
    on<ClientCreateSubmitted>(_onSubmitted);
  }

  final ClientsRepository _clientsRepository;

  Future<void> _onSubmitted(ClientCreateSubmitted event, Emitter<ClientCreateState> emit) async {
    emit(state.copyWith(status: ClientCreateStatus.submitting));

    try {
      await _clientsRepository.createClient(state.form);
      emit(state.copyWith(status: ClientCreateStatus.success));
    } on ApiException catch (error) {
      emit(state.copyWith(status: ClientCreateStatus.failure, errorMessage: error.message));
    }
  }
}
