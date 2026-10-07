enum UserRole {
    Admin,
    Guest { invited_by: UserId, since: DateTime },
    Member,
}
