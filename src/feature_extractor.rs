use struct_iterable::Iterable;

use crate::hachi;
use crate::game;
use crate::static_features;

#[derive(Iterable)]
pub struct Features {
    pub heights:[u32;10],
    pub height_differences:[i32;9],
    pub first_hole_depths:[i32;10],
    pub piece_distance:[i32;7],
    pub piece_counts:[i32;7],
    pub hold_or_current_onehot:[i32;7],
    pub next_onehot:[i32;7],
    pub all_3x3s:[i32;512],

    pub sunbeam_max_height:u32,
    pub sunbeam_bumpiness:i32,
    pub sunbeam_well_x:usize,
    pub sunbeam_well_depth:i32,
    pub sunbeam_max_donated_height:u32,
    pub sunbeam_n_donations:i32,
    pub sunbeam_t_clears:[i32;4],

    pub cc_holes:i32,
    pub cc_coveredness:i32,
    pub cc_row_transitions:i32
}

pub fn extract_features(game: &game::GameState) -> Features {
    let sf = static_features::get_static_features(&game);
    let hf = hachi::get_hachi_features(&game);

    Features {
        heights: hf.heights,
        height_differences: hf.height_differences,
        first_hole_depths: hf.first_hole_depths,
        piece_distance: hf.piece_distance,
        piece_counts: hf.piece_counts,
        hold_or_current_onehot: hf.hold_or_current_onehot,
        next_onehot: hf.next_onehot,
        all_3x3s: hf.all_3x3s,

        sunbeam_max_height: sf.sunbeam_max_height,
        sunbeam_bumpiness: sf.sunbeam_bumpiness,
        sunbeam_well_x: sf.sunbeam_well_x,
        sunbeam_well_depth: sf.sunbeam_well_depth,
        sunbeam_max_donated_height: sf.sunbeam_max_donated_height,
        sunbeam_n_donations: sf.sunbeam_n_donations,
        sunbeam_t_clears: sf.sunbeam_t_clears,

        cc_holes: sf.cc_holes,
        cc_coveredness: sf.cc_coveredness,
        cc_row_transitions: sf.cc_row_transitions,
    }
}

impl Features {
    pub fn sql_columns(prefix: &str) -> String {
        Self::sql_columns_with_options(prefix, false)
    }
    
    pub fn sql_columns_with_types(prefix: &str) -> String {
        Self::sql_columns_with_options(prefix, true)
    }
    
    fn sql_columns_with_options(prefix: &str, include_types: bool) -> String {
        let mut columns = Vec::new();
        
        let type_suffix = if include_types { " INTEGER NOT NULL" } else { "" };
        
        // heights array
        for i in 0..10 {
            columns.push(format!("{}_{}{}{}", prefix, "heights", i, type_suffix));
        }
        
        // height_differences array
        for i in 0..9 {
            columns.push(format!("{}_{}{}{}", prefix, "height_differences", i, type_suffix));
        }
        
        // first_hole_depths array
        for i in 0..10 {
            columns.push(format!("{}_{}{}{}", prefix, "first_hole_depths", i, type_suffix));
        }
        
        // piece_distance array
        for i in 0..7 {
            columns.push(format!("{}_{}{}{}", prefix, "piece_distance", i, type_suffix));
        }
        
        // piece_counts array
        for i in 0..7 {
            columns.push(format!("{}_{}{}{}", prefix, "piece_counts", i, type_suffix));
        }
        
        // hold_or_current_onehot array
        for i in 0..7 {
            columns.push(format!("{}_{}{}{}", prefix, "hold_or_current_onehot", i, type_suffix));
        }
        
        // next_onehot array
        for i in 0..7 {
            columns.push(format!("{}_{}{}{}", prefix, "next_onehot", i, type_suffix));
        }
        
        // all_3x3s array
        for i in 0..512 {
            columns.push(format!("{}_{}{}{}", prefix, "all_3x3s", i, type_suffix));
        }
        
        // sunbeam scalar fields
        columns.push(format!("{}_{}{}", prefix, "sunbeam_max_height", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "sunbeam_bumpiness", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "sunbeam_well_x", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "sunbeam_well_depth", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "sunbeam_max_donated_height", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "sunbeam_n_donations", type_suffix));
        
        // sunbeam_t_clears array
        for i in 0..4 {
            columns.push(format!("{}_{}{}{}", prefix, "sunbeam_t_clears", i, type_suffix));
        }
        
        // cc scalar fields
        columns.push(format!("{}_{}{}", prefix, "cc_holes", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "cc_coveredness", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "cc_row_transitions", type_suffix));
        
        columns.join(", ")
    }

    pub fn sql_placeholders() -> String {
        let count = 
                    10 + 9 + 10 + 7 + 7 + 7 + 7 + 512 +  // arrays
                    6 +  // sunbeam scalars
                    4 +  // sunbeam_t_clears
                    3;   // cc scalars
        
        vec!["?"; count].join(", ")
    }
}

impl Features {
    pub fn values(&self) -> Vec<rusqlite::types::Value> {
        let mut vals = Vec::new();
        
        // heights array
        for &h in &self.heights {
            vals.push(rusqlite::types::Value::from(h as i64));
        }
        
        // height_differences array
        for &hd in &self.height_differences {
            vals.push(rusqlite::types::Value::from(hd as i64));
        }
        
        // first_hole_depths array
        for &fhd in &self.first_hole_depths {
            vals.push(rusqlite::types::Value::from(fhd as i64));
        }
        
        // piece_distance array
        for &pd in &self.piece_distance {
            vals.push(rusqlite::types::Value::from(pd as i64));
        }
        
        // piece_counts array
        for &pc in &self.piece_counts {
            vals.push(rusqlite::types::Value::from(pc as i64));
        }
        
        // hold_or_current_onehot array
        for &hoc in &self.hold_or_current_onehot {
            vals.push(rusqlite::types::Value::from(hoc as i64));
        }
        
        // next_onehot array
        for &no in &self.next_onehot {
            vals.push(rusqlite::types::Value::from(no as i64));
        }

        // 3x3s
        for &no in &self.all_3x3s {
            vals.push(rusqlite::types::Value::from(no as i64));
        }
        
        
        // sunbeam scalar fields
        vals.push(rusqlite::types::Value::from(self.sunbeam_max_height as i64));
        vals.push(rusqlite::types::Value::from(self.sunbeam_bumpiness as i64));
        vals.push(rusqlite::types::Value::from(self.sunbeam_well_x as i64));
        vals.push(rusqlite::types::Value::from(self.sunbeam_well_depth as i64));
        vals.push(rusqlite::types::Value::from(self.sunbeam_max_donated_height as i64));
        vals.push(rusqlite::types::Value::from(self.sunbeam_n_donations as i64));
        
        // sunbeam_t_clears array
        for &tc in &self.sunbeam_t_clears {
            vals.push(rusqlite::types::Value::from(tc as i64));
        }
        
        // cc scalar fields
        vals.push(rusqlite::types::Value::from(self.cc_holes as i64));
        vals.push(rusqlite::types::Value::from(self.cc_coveredness as i64));
        vals.push(rusqlite::types::Value::from(self.cc_row_transitions as i64));
        
        vals
    }
}