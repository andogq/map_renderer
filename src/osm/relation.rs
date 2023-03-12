use super::Tags;

pub enum RelationMemberType {
    Node,
    Way,
    Relation,
}

impl From<osmpbf::RelMemberType> for RelationMemberType {
    fn from(member_type: osmpbf::RelMemberType) -> Self {
        use RelationMemberType::*;

        match member_type {
            osmpbf::RelMemberType::Node => Node,
            osmpbf::RelMemberType::Way => Way,
            osmpbf::RelMemberType::Relation => Relation,
        }
    }
}

pub struct RelationMember {
    pub role: Option<String>,
    pub id: i64,
    pub member_type: RelationMemberType,
}

impl From<osmpbf::RelMember<'_>> for RelationMember {
    fn from(member: osmpbf::RelMember<'_>) -> Self {
        Self {
            role: member.role().ok().map(|role| role.to_string()),
            id: member.member_id,
            member_type: member.member_type.into(),
        }
    }
}

pub struct Relation {
    pub tags: Tags,
    pub members: Vec<RelationMember>,
}

impl From<osmpbf::Relation<'_>> for Relation {
    fn from(relation: osmpbf::Relation<'_>) -> Self {
        Self {
            members: relation.members().map(|m| m.into()).collect(),
            tags: relation.tags().into(),
        }
    }
}
