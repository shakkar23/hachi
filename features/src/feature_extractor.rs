use crate::hachi;
use crate::game;
use crate::static_features;
use crate::whitelist;

pub struct Features {
    pub heights:[u32;10],
    pub height_differences:[i16;9],
    pub first_hole_depths:[i16;10],
    pub garbage_holes:[i16;20],
    pub piece_distance:[i16;7],
    pub piece_counts:[i16;7],
    pub hold_or_current_onehot:[i16;7],
    pub next_onehot:[i16;7],
    pub all_3x3s:[i16;512],
    pub all_3x3s_with_x:[i16;512],
    pub all_3x3s_with_y:[i16;512],
    pub meter: i16,
    pub combo: i16,
    pub b2b: i16,

    pub sunbeam_max_height:u32,
    pub sunbeam_bumpiness:i16,
    pub sunbeam_well_x:usize,
    pub sunbeam_well_depth:i16,
    pub sunbeam_max_donated_height:u32,
    pub sunbeam_n_donations:i16,
    pub sunbeam_t_clears:[i16;4],

    pub cc_holes:i16,
    pub cc_coveredness:i16,
    pub cc_row_transitions:i16
}

pub struct Row {
    pub game_id:     u16,
    pub move_index:  u16,
    pub state:       game::State,
    pub ground_truth: f32,
    pub features:    (Features, Features),
}


pub fn extract_features(game: &game::GameState) -> Features {
    let sf = static_features::get_static_features(&game);
    let hf = hachi::get_hachi_features(&game);

    Features {
        heights: hf.heights,
        height_differences: hf.height_differences,
        first_hole_depths: hf.first_hole_depths,
        garbage_holes: hf.garbage_holes,
        piece_distance: hf.piece_distance,
        piece_counts: hf.piece_counts,
        hold_or_current_onehot: hf.hold_or_current_onehot,
        next_onehot: hf.next_onehot,
        all_3x3s: hf.all_3x3s,
        all_3x3s_with_x: hf.all_3x3s_with_x,
        all_3x3s_with_y: hf.all_3x3s_with_y,
        meter: hf.meter,
        combo: hf.combo,
        b2b: hf.b2b,

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

const use_3x3s:bool = true;
const use_positional_3x3s:bool = true;

impl Features {
    pub fn sql_columns(prefix: &str) -> String {
        Self::sql_columns_with_options(prefix, false)
    }
    
    pub fn sql_columns_with_types(prefix: &str) -> String {
        Self::sql_columns_with_options(prefix, true)
    }
    
    fn sql_columns_with_options(prefix: &str, include_types: bool) -> String {
        let mut columns = Vec::new();
        
        let type_suffix = if include_types { " SMALLINT NOT NULL" } else { "" };
        
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

        for i in 0..20 {
            columns.push(format!("{}_{}{}{}", prefix, "garbage_holes", i, type_suffix));
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
        
        if use_3x3s {
            // all_3x3s array
            for i in whitelist::top_100_3x3s.iter() {
                columns.push(format!("{}_{}{}{}", prefix, "all_3x3s", i, type_suffix));
            }
        }
        
        if use_positional_3x3s {
            // all_3x3s_with_x array
            for i in whitelist::top_100_3x3s_with_x.iter() {
                columns.push(format!("{}_{}{}{}", prefix, "all_3x3s_with_x", i, type_suffix));
            }

            // all_3x3s_with_y array
            for i in whitelist::top_100_3x3s_with_y.iter() {
                columns.push(format!("{}_{}{}{}", prefix, "all_3x3s_with_y", i, type_suffix));
            }
        }

        // hachi scalar
        columns.push(format!("{}_{}{}", prefix, "meter", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "combo", type_suffix));
        columns.push(format!("{}_{}{}", prefix, "b2b", type_suffix));

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

    pub const count:usize =
        10 + // heights
        9 + // height differences
        10 + // first hole depths
        20 + // garbage holes
        7 + // piece_distance
        7 +  // piece counts
        7 +  // hold or current
        7 +  // next
        if use_3x3s {whitelist::top_100_3x3s.len()} else {0} +  // 3x3s
        if use_positional_3x3s {whitelist::top_100_3x3s_with_x.len()} else {0} + // 3x3s with x
        if use_positional_3x3s {whitelist::top_100_3x3s_with_y.len()} else {0} + // 3x3s with y
        3 + // hachi scalars
        6 +  // sunbeam scalars
        4 +  // sunbeam_t_clears
        3   // cc scalars
    ;

    pub fn sql_placeholders() -> String {
        
        vec!["?"; Features::count].join(", ")
    }
}

impl Features {
    pub fn values(&self) -> Vec<i16> {
        let mut vals = Vec::new();
        
        // heights array
        for &h in &self.heights {
            vals.push(h as i16);
        }
        
        // height_differences array
        for &hd in &self.height_differences {
            vals.push(hd as i16);
        }
        
        // first_hole_depths array
        for &fhd in &self.first_hole_depths {
            vals.push(fhd as i16);
        }
        
        // garbage_holes array
        for &gh in &self.garbage_holes {
            vals.push(gh as i16);
        }

        // piece_distance array
        for &pd in &self.piece_distance {
            vals.push(pd as i16);
        }
        
        // piece_counts array
        for &pc in &self.piece_counts {
            vals.push(pc as i16);
        }
        
        // hold_or_current_onehot array
        for &hoc in &self.hold_or_current_onehot {
            vals.push(hoc as i16);
        }
        
        // next_onehot array
        for &no in &self.next_onehot {
            vals.push(no as i16);
        }

        if use_3x3s {
            // 3x3s
            for i in whitelist::top_100_3x3s.iter() {
                vals.push(self.all_3x3s[*i] as i16);
            }
        }

        if use_positional_3x3s {

            // 3x3s with x
            for i in whitelist::top_100_3x3s_with_x.iter() {
                vals.push(self.all_3x3s_with_x[*i] as i16);
            }
            
            // 3x3s with y
            for i in whitelist::top_100_3x3s_with_y.iter() {
                vals.push(self.all_3x3s_with_y[*i] as i16);
            }
        }

        vals.push(self.meter as i16);
        vals.push(self.combo as i16);
        vals.push(self.b2b as i16);
        
        // sunbeam scalar fields
        vals.push(self.sunbeam_max_height as i16);
        vals.push(self.sunbeam_bumpiness as i16);
        vals.push(self.sunbeam_well_x as i16);
        vals.push(self.sunbeam_well_depth as i16);
        vals.push(self.sunbeam_max_donated_height as i16);
        vals.push(self.sunbeam_n_donations as i16);
        
        // sunbeam_t_clears array
        for &tc in &self.sunbeam_t_clears {
            vals.push(tc as i16);
        }
        
        // cc scalar fields
        vals.push(self.cc_holes as i16);
        vals.push(self.cc_coveredness as i16);
        vals.push(self.cc_row_transitions as i16);
        
        vals
    }
}