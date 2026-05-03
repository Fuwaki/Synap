use std::collections::{HashMap, HashSet, VecDeque};

use uuid::Uuid;

use crate::{
    dto::{
        NoteNeighborContextDTO, NoteNeighborsDTO, NoteSegmentBranchChoiceDTO, NoteSegmentDTO,
        NoteSegmentDirectionDTO, NoteSegmentStepDTO,
    },
    error::NoteError,
    models::note::NoteReader,
    views::note_view::NoteView,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteSegmentDirection {
    Forward,
    Backward,
}

pub struct NoteSegmentView<'a, 'b: 'a> {
    reader: &'a NoteReader<'b>,
    anchor_id: Uuid,
    direction: NoteSegmentDirection,
    forward_weights: HashMap<Uuid, u32>,
    backward_weights: HashMap<Uuid, u32>,
}

impl<'a, 'b> NoteSegmentView<'a, 'b> {
    pub fn new(
        reader: &'a NoteReader<'b>,
        anchor_id: Uuid,
        direction: NoteSegmentDirection,
    ) -> Result<Self, NoteError> {
        reader
            .get_ref_by_id(&anchor_id)
            .map_err(NoteError::Db)?
            .ok_or(NoteError::IdNotFound { id: anchor_id })?;

        let connected_ids = collect_connected_ids(reader, anchor_id)?;
        let forward_adj = build_adjacency(reader, &connected_ids, NoteSegmentDirection::Forward)?;
        let backward_adj = build_adjacency(reader, &connected_ids, NoteSegmentDirection::Backward)?;
        let forward_weights = longest_path_weights(&forward_adj);
        let backward_weights = longest_path_weights(&backward_adj);

        Ok(Self {
            reader,
            anchor_id,
            direction,
            forward_weights,
            backward_weights,
        })
    }

    pub fn to_dto(&self) -> Result<NoteSegmentDTO, NoteError> {
        let mut steps = Vec::new();
        let mut current_id = self.anchor_id;
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(current_id) {
                break;
            }

            let note = self
                .reader
                .get_by_id(&current_id)
                .map_err(NoteError::Db)?
                .ok_or(NoteError::IdNotFound { id: current_id })?;
            let note_dto = NoteView::new(self.reader, note).to_dto()?;

            let next_choices = self.branch_choices(current_id, self.direction)?;
            let prev_choices = self.branch_choices(current_id, self.direction.reverse())?;
            let stops_here = self.should_stop_here(&next_choices, &prev_choices);

            let next_id = if stops_here || next_choices.len() != 1 {
                None
            } else {
                Some(parse_note_id(&next_choices[0].note.id)?)
            };

            steps.push(NoteSegmentStepDTO {
                note: note_dto,
                next_choices,
                prev_choices,
                stops_here,
            });

            match next_id {
                Some(id) => current_id = id,
                None => break,
            }
        }

        Ok(NoteSegmentDTO {
            anchor_id: self.anchor_id.to_string(),
            direction: self.direction.into(),
            steps,
        })
    }

    pub fn neighbors_dto(&self, id: Uuid) -> Result<NoteNeighborsDTO, NoteError> {
        let note = self
            .reader
            .get_by_id(&id)
            .map_err(NoteError::Db)?
            .ok_or(NoteError::IdNotFound { id })?;
        let parents = self.branch_choices(id, NoteSegmentDirection::Backward)?;
        let children = self.branch_choices(id, NoteSegmentDirection::Forward)?;
        let parent_contexts = parents
            .iter()
            .map(|choice| self.neighbor_context_dto(choice, id))
            .collect::<Result<Vec<_>, _>>()?;
        let child_contexts = children
            .iter()
            .map(|choice| self.neighbor_context_dto(choice, id))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(NoteNeighborsDTO {
            note: NoteView::new(self.reader, note).to_dto()?,
            parents,
            children,
            parent_contexts,
            child_contexts,
        })
    }

    fn neighbor_context_dto(
        &self,
        choice: &NoteSegmentBranchChoiceDTO,
        center_id: Uuid,
    ) -> Result<NoteNeighborContextDTO, NoteError> {
        let center_id = center_id.to_string();
        let choice_id = parse_note_id(&choice.note.id)?;
        let parents = self
            .branch_choices(choice_id, NoteSegmentDirection::Backward)?
            .into_iter()
            .filter(|item| item.note.id != center_id)
            .collect();
        let children = self
            .branch_choices(choice_id, NoteSegmentDirection::Forward)?
            .into_iter()
            .filter(|item| item.note.id != center_id)
            .collect();

        Ok(NoteNeighborContextDTO {
            note: choice.note.clone(),
            weight: choice.weight,
            parents,
            children,
        })
    }

    fn branch_choices(
        &self,
        id: Uuid,
        direction: NoteSegmentDirection,
    ) -> Result<Vec<NoteSegmentBranchChoiceDTO>, NoteError> {
        let mut choices = match direction {
            NoteSegmentDirection::Forward => self.collect_live_neighbor_ids(id, direction)?,
            NoteSegmentDirection::Backward => self.collect_live_neighbor_ids(id, direction)?,
        };

        choices.sort_by(|left, right| {
            let left_weight = self.weight_for(*left, direction);
            let right_weight = self.weight_for(*right, direction);
            right_weight.cmp(&left_weight).then_with(|| left.cmp(right))
        });

        choices
            .into_iter()
            .map(|choice_id| {
                let note = self
                    .reader
                    .get_by_id(&choice_id)
                    .map_err(NoteError::Db)?
                    .ok_or(NoteError::IdNotFound { id: choice_id })?;
                let note_dto = NoteView::new(self.reader, note).to_dto()?;
                Ok(NoteSegmentBranchChoiceDTO {
                    note: note_dto,
                    weight: self.weight_for(choice_id, direction),
                })
            })
            .collect()
    }

    fn should_stop_here(
        &self,
        next_choices: &[NoteSegmentBranchChoiceDTO],
        prev_choices: &[NoteSegmentBranchChoiceDTO],
    ) -> bool {
        next_choices.len() != 1 || prev_choices.len() > 1
    }

    fn collect_live_neighbor_ids(
        &self,
        id: Uuid,
        direction: NoteSegmentDirection,
    ) -> Result<Vec<Uuid>, NoteError> {
        let mut result = Vec::new();
        for neighbor_id in collect_neighbor_ids(self.reader, id, direction)? {
            let note_ref = match self
                .reader
                .get_ref_by_id(&neighbor_id)
                .map_err(NoteError::Db)?
            {
                Some(note_ref) => note_ref,
                None => return Err(NoteError::IdNotFound { id: neighbor_id }),
            };

            if !note_ref.is_deleted() {
                result.push(neighbor_id);
            }
        }

        result.sort_unstable();
        result.dedup();
        Ok(result)
    }

    fn weight_for(&self, id: Uuid, direction: NoteSegmentDirection) -> u32 {
        match direction {
            NoteSegmentDirection::Forward => *self.forward_weights.get(&id).unwrap_or(&1),
            NoteSegmentDirection::Backward => *self.backward_weights.get(&id).unwrap_or(&1),
        }
    }
}

impl NoteSegmentDirection {
    fn reverse(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

impl From<NoteSegmentDirection> for NoteSegmentDirectionDTO {
    fn from(value: NoteSegmentDirection) -> Self {
        match value {
            NoteSegmentDirection::Forward => Self::Forward,
            NoteSegmentDirection::Backward => Self::Backward,
        }
    }
}

fn parse_note_id(value: &str) -> Result<Uuid, NoteError> {
    Uuid::parse_str(value).map_err(|_| NoteError::InvalidTitle("invalid note id".to_string()))
}

fn collect_connected_ids(
    reader: &NoteReader<'_>,
    anchor_id: Uuid,
) -> Result<HashSet<Uuid>, NoteError> {
    let mut seen = HashSet::from([anchor_id]);
    let mut queue = VecDeque::from([anchor_id]);

    while let Some(current_id) = queue.pop_front() {
        for direction in [
            NoteSegmentDirection::Forward,
            NoteSegmentDirection::Backward,
        ] {
            for neighbor_id in collect_neighbor_ids(reader, current_id, direction)? {
                if seen.insert(neighbor_id) {
                    queue.push_back(neighbor_id);
                }
            }
        }
    }

    Ok(seen)
}

fn build_adjacency(
    reader: &NoteReader<'_>,
    ids: &HashSet<Uuid>,
    direction: NoteSegmentDirection,
) -> Result<HashMap<Uuid, Vec<Uuid>>, NoteError> {
    let mut adjacency = HashMap::new();

    for id in ids {
        let mut neighbors = Vec::new();
        for neighbor_id in collect_neighbor_ids(reader, *id, direction)? {
            if ids.contains(&neighbor_id) {
                neighbors.push(neighbor_id);
            }
        }

        neighbors.sort_unstable();
        neighbors.dedup();
        adjacency.insert(*id, neighbors);
    }

    Ok(adjacency)
}

fn collect_neighbor_ids(
    reader: &NoteReader<'_>,
    id: Uuid,
    direction: NoteSegmentDirection,
) -> Result<Vec<Uuid>, NoteError> {
    match direction {
        NoteSegmentDirection::Forward => reader
            .children_raw(&id)
            .map_err(|e| NoteError::Db(e.into()))?
            .map(|item| item.map_err(|e| NoteError::Db(e.into())))
            .collect(),
        NoteSegmentDirection::Backward => reader
            .parents_raw(&id)
            .map_err(|e| NoteError::Db(e.into()))?
            .map(|item| item.map_err(|e| NoteError::Db(e.into())))
            .collect(),
    }
}

fn longest_path_weights(adjacency: &HashMap<Uuid, Vec<Uuid>>) -> HashMap<Uuid, u32> {
    let mut indegree = HashMap::new();
    for (&id, neighbors) in adjacency {
        indegree.entry(id).or_insert(0_u32);
        for neighbor in neighbors {
            *indegree.entry(*neighbor).or_insert(0) += 1;
        }
    }

    let mut queue = VecDeque::new();
    for (&id, &degree) in &indegree {
        if degree == 0 {
            queue.push_back(id);
        }
    }

    let mut topo = Vec::with_capacity(indegree.len());
    while let Some(id) = queue.pop_front() {
        topo.push(id);
        if let Some(neighbors) = adjacency.get(&id) {
            for neighbor in neighbors {
                if let Some(degree) = indegree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
    }

    let mut weights = HashMap::new();
    for id in topo.into_iter().rev() {
        let best_child = adjacency
            .get(&id)
            .into_iter()
            .flatten()
            .filter_map(|neighbor| weights.get(neighbor).copied())
            .max()
            .unwrap_or(0);
        weights.insert(id, best_child + 1);
    }

    weights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::note::Note;
    use redb::{Database, ReadableDatabase};
    use tempfile::NamedTempFile;

    fn create_temp_db() -> Database {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::create(temp_file.path()).unwrap();
        let write_txn = db.begin_write().unwrap();
        Note::init_schema(&write_txn).unwrap();
        crate::models::tag::TagWriter::init_schema(&write_txn).unwrap();
        write_txn.commit().unwrap();
        db
    }

    #[test]
    fn segment_stops_before_branch_and_returns_weighted_choices() {
        let db = create_temp_db();

        let write_txn = db.begin_write().unwrap();
        let a = Note::create(&write_txn, "a".to_string(), vec![]).unwrap();
        let b = Note::create(&write_txn, "b".to_string(), vec![]).unwrap();
        let c = Note::create(&write_txn, "c".to_string(), vec![]).unwrap();
        let d = Note::create(&write_txn, "d".to_string(), vec![]).unwrap();
        let e = Note::create(&write_txn, "e".to_string(), vec![]).unwrap();
        a.reply(&write_txn, &b).unwrap();
        b.reply(&write_txn, &c).unwrap();
        b.reply(&write_txn, &d).unwrap();
        d.reply(&write_txn, &e).unwrap();
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let reader = NoteReader::new(&read_txn).unwrap();
        let view =
            NoteSegmentView::new(&reader, a.get_id(), NoteSegmentDirection::Forward).unwrap();
        let dto = view.to_dto().unwrap();

        assert_eq!(dto.steps.len(), 2);
        assert_eq!(dto.steps[0].note.id, a.get_id().to_string());
        assert_eq!(dto.steps[1].note.id, b.get_id().to_string());
        assert!(dto.steps[1].stops_here);
        assert_eq!(dto.steps[1].next_choices.len(), 2);
        assert_eq!(dto.steps[1].next_choices[0].note.id, d.get_id().to_string());
        assert_eq!(dto.steps[1].next_choices[0].weight, 2);
        assert_eq!(dto.steps[1].next_choices[1].note.id, c.get_id().to_string());
        assert_eq!(dto.steps[1].next_choices[1].weight, 1);
    }

    #[test]
    fn segment_stops_at_anchor_branch_and_does_not_expand_grandchildren() {
        let db = create_temp_db();

        let write_txn = db.begin_write().unwrap();
        let a = Note::create(&write_txn, "a".to_string(), vec![]).unwrap();
        let b = Note::create(&write_txn, "b".to_string(), vec![]).unwrap();
        let c = Note::create(&write_txn, "c".to_string(), vec![]).unwrap();
        let d = Note::create(&write_txn, "d".to_string(), vec![]).unwrap();
        a.reply(&write_txn, &b).unwrap();
        a.reply(&write_txn, &c).unwrap();
        b.reply(&write_txn, &d).unwrap();
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let reader = NoteReader::new(&read_txn).unwrap();
        let view =
            NoteSegmentView::new(&reader, a.get_id(), NoteSegmentDirection::Forward).unwrap();
        let dto = view.to_dto().unwrap();

        assert_eq!(dto.steps.len(), 1);
        assert_eq!(dto.steps[0].note.id, a.get_id().to_string());
        assert!(dto.steps[0].stops_here);
        assert_eq!(dto.steps[0].next_choices.len(), 2);
        assert!(dto.steps[0]
            .next_choices
            .iter()
            .any(|choice| choice.note.id == b.get_id().to_string()));
        assert!(dto.steps[0]
            .next_choices
            .iter()
            .any(|choice| choice.note.id == c.get_id().to_string()));
        assert!(!dto.steps[0]
            .next_choices
            .iter()
            .any(|choice| choice.note.id == d.get_id().to_string()));
    }

    #[test]
    fn segment_stops_before_merge_when_iterating_backward() {
        let db = create_temp_db();

        let write_txn = db.begin_write().unwrap();
        let a = Note::create(&write_txn, "a".to_string(), vec![]).unwrap();
        let b = Note::create(&write_txn, "b".to_string(), vec![]).unwrap();
        let c = Note::create(&write_txn, "c".to_string(), vec![]).unwrap();
        let d = Note::create(&write_txn, "d".to_string(), vec![]).unwrap();
        a.reply(&write_txn, &c).unwrap();
        b.reply(&write_txn, &c).unwrap();
        c.reply(&write_txn, &d).unwrap();
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let reader = NoteReader::new(&read_txn).unwrap();
        let view =
            NoteSegmentView::new(&reader, d.get_id(), NoteSegmentDirection::Backward).unwrap();
        let dto = view.to_dto().unwrap();

        assert_eq!(dto.steps.len(), 2);
        assert_eq!(dto.steps[0].note.id, d.get_id().to_string());
        assert_eq!(dto.steps[1].note.id, c.get_id().to_string());
        assert!(dto.steps[1].stops_here);
        assert_eq!(dto.steps[1].next_choices.len(), 2);
    }

    #[test]
    fn neighbors_returns_true_parents_and_children_independent_of_segment_direction() {
        let db = create_temp_db();

        let write_txn = db.begin_write().unwrap();
        let a_prime = Note::create(&write_txn, "a'".to_string(), vec![]).unwrap();
        let a = Note::create(&write_txn, "a".to_string(), vec![]).unwrap();
        let b_prime = Note::create(&write_txn, "b'".to_string(), vec![]).unwrap();
        let b = Note::create(&write_txn, "b".to_string(), vec![]).unwrap();
        let c = Note::create(&write_txn, "c".to_string(), vec![]).unwrap();
        a_prime.reply(&write_txn, &a).unwrap();
        b_prime.reply(&write_txn, &b).unwrap();
        b.reply(&write_txn, &a).unwrap();
        a.reply(&write_txn, &c).unwrap();
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let reader = NoteReader::new(&read_txn).unwrap();
        let view =
            NoteSegmentView::new(&reader, b.get_id(), NoteSegmentDirection::Forward).unwrap();
        let dto = view.neighbors_dto(a.get_id()).unwrap();

        assert_eq!(dto.note.id, a.get_id().to_string());
        assert_eq!(dto.parents.len(), 2);
        assert!(dto
            .parents
            .iter()
            .any(|choice| choice.note.id == a_prime.get_id().to_string()));
        assert!(dto
            .parents
            .iter()
            .any(|choice| choice.note.id == b.get_id().to_string()));
        assert_eq!(dto.children.len(), 1);
        assert_eq!(dto.children[0].note.id, c.get_id().to_string());
        let b_context = dto
            .parent_contexts
            .iter()
            .find(|context| context.note.id == b.get_id().to_string())
            .unwrap();
        assert_eq!(b_context.parents.len(), 1);
        assert_eq!(b_context.parents[0].note.id, b_prime.get_id().to_string());
        let c_context = dto
            .child_contexts
            .iter()
            .find(|context| context.note.id == c.get_id().to_string())
            .unwrap();
        assert!(c_context.parents.is_empty());
        assert!(c_context.children.is_empty());
    }
}
