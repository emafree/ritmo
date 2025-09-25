pub trait FromDto<DTO> {
    fn from_dto(dto: &DTO) -> Self;
}

pub trait FromModel<Model> {
    fn from_model(model: &Model) -> Self;
}
