part of 'client_detail_bloc.dart';

enum ClientDetailStatus { loading, loaded, saving, deleting, failure, deleted }

class ClientDetailState extends Equatable {
  const ClientDetailState({
    this.status = ClientDetailStatus.loading,
    this.client,
    this.form = const ClientInput.blank(),
    this.editing = false,
    this.errorMessage,
  });

  final ClientDetailStatus status;
  final Client? client;
  final ClientInput form;
  final bool editing;
  final String? errorMessage;

  ClientDetailState copyWith({
    ClientDetailStatus? status,
    Client? client,
    ClientInput? form,
    bool? editing,
    String? errorMessage,
  }) {
    return ClientDetailState(
      status: status ?? this.status,
      client: client ?? this.client,
      form: form ?? this.form,
      editing: editing ?? this.editing,
      errorMessage: errorMessage,
    );
  }

  @override
  List<Object?> get props => [status, client, form, editing, errorMessage];
}
