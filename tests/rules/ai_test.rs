use crate::common::{test_ai, test_ai_engine, test_config, test_rule};
use std::path::Path;

// ==========================================
// 1-10: Domain Specific Categories
// ==========================================

#[test]
fn test_ai_1_music() {
    assert!(
        test_ai(
            "lofi_hiphop_beat_music_track.mp3",
            "music, audio, songs, soundtrack"
        )
        .is_some()
    );
}

#[test]
fn test_ai_2_video() {
    assert!(
        test_ai(
            "wedding_ceremony_4k_video_footage.mp4",
            "videos, movies, cinema, footage"
        )
        .is_some()
    );
}

#[test]
fn test_ai_3_code() {
    assert!(
        test_ai(
            "user_authentication_logic_script.py",
            "programming code, source files, script"
        )
        .is_some()
    );
}

#[test]
fn test_ai_4_books() {
    assert!(
        test_ai(
            "clean_code_architecture_book_reading.pdf",
            "books, ebooks, literature, reading material"
        )
        .is_some()
    );
}

#[test]
fn test_ai_5_games() {
    assert!(
        test_ai(
            "elden_ring_video_game_save_data.bin",
            "video games, gaming assets, playstation"
        )
        .is_some()
    );
}

#[test]
fn test_ai_6_food() {
    assert!(
        test_ai(
            "thai_green_curry_recipe_instructions.txt",
            "food recipes, cooking instructions, kitchen"
        )
        .is_some()
    );
}

#[test]
fn test_ai_7_health() {
    assert!(
        test_ai(
            "blood_test_results_medical_doctor_health.pdf",
            "medical records, health reports, doctor"
        )
        .is_some()
    );
}

#[test]
fn test_ai_8_sports() {
    assert!(
        test_ai(
            "marathon_training_plan_fitness_exercise.xlsx",
            "sports, fitness, exercise, workout"
        )
        .is_some()
    );
}

#[test]
fn test_ai_9_art() {
    assert!(
        test_ai(
            "abstract_painting_concept_creative_design.psd",
            "art, design, illustration, creative work"
        )
        .is_some()
    );
}

#[test]
fn test_ai_10_education() {
    assert!(
        test_ai(
            "quantum_physics_lecture_university_study_notes.docx",
            "education, study materials, university"
        )
        .is_some()
    );
}

// ==========================================
// 11-20: Edge Cases (Structure & Content)
// ==========================================

#[test]
fn test_ai_11_edge_short_name() {
    assert!(test_ai("cv_resume.pdf", "resume, curriculum vitae, job application").is_some());
}

#[test]
fn test_ai_12_edge_long_name() {
    let long_name = "a".repeat(200) + "_financial_billing_invoice.pdf";
    assert!(test_ai(&long_name, "financial invoice, billing").is_some());
}

#[test]
fn test_ai_13_edge_unicode_emoji() {
    assert!(
        test_ai(
            "💰_financial_money_report_🔥.xlsx",
            "financial report, money, profit"
        )
        .is_some()
    );
}

#[test]
fn test_ai_14_edge_no_extension() {
    assert!(
        test_ai(
            "README_INSTALLATION_GUIDE_INSTRUCTIONS",
            "documentation, instructions, guide"
        )
        .is_some()
    );
}

#[test]
fn test_ai_15_edge_multiple_dots() {
    assert!(
        test_ai(
            "archive.backup.2024.v1.zip",
            "compressed archives, backup files"
        )
        .is_some()
    );
}

#[test]
fn test_ai_16_edge_only_numbers() {
    assert!(
        test_ai(
            "20241225_calendar_dates_records.pdf",
            "calendar, dates, records"
        )
        .is_some()
    );
}

#[test]
fn test_ai_17_edge_only_symbols() {
    assert!(test_ai("!@#$%^&.txt", "random symbols, text").is_none());
}

#[test]
fn test_ai_18_logic_empty_label() {
    let config = test_config(Path::new("."));
    let rule = test_rule("Empty", vec![], vec![]);
    let rules = vec![rule];
    let mut ai = test_ai_engine(&rules).unwrap();
    assert!(ai.determine_rule("anything.txt", &config, &rules).is_none());
}

#[test]
fn test_ai_19_logic_identical_labels() {
    let config = test_config(Path::new("."));
    let mut r1 = test_rule("Rule1", vec![], vec![]);
    r1.semantic_label = Some("identical label text content".to_string());
    let mut r2 = test_rule("Rule2", vec![], vec![]);
    r2.semantic_label = Some("identical label text content".to_string());
    let rules = vec![r1, r2];
    let mut ai = test_ai_engine(&rules).unwrap();
    let res = ai.determine_rule("identical_label_text_content.txt", &config, &rules);
    assert_eq!(res.unwrap().name, "Rule1");
}

#[test]
fn test_ai_20_logic_priority_mixed() {
    assert!(test_ai("travel_vacation_paris_trip.jpg", "travel, vacation, trip").is_some());
}

// ==========================================
// 21-30: Huge, Performance & Deep Context
// ==========================================

#[test]
fn test_ai_21_massive_rules() {
    let config = test_config(Path::new("."));
    let subjects = [
        "Mathematics",
        "Biology",
        "History",
        "Physics",
        "Chemistry",
        "Geography",
        "Literature",
        "Art",
        "Music",
        "Physical Education",
        "Computer Science",
        "Economics",
        "Psychology",
        "Sociology",
        "Philosophy",
        "Politics",
        "Law",
        "Medicine",
        "Engineering",
        "Architecture",
    ];
    let mut rules = vec![];
    for sub in &subjects {
        let mut r = test_rule(sub, vec![], vec![]);
        r.semantic_label = Some(format!("educational material about {}", sub));
        rules.push(r);
    }
    let mut ai = test_ai_engine(&rules).unwrap();
    let res = ai.determine_rule("advanced_physics_quantum_mechanics.pdf", &config, &rules);
    assert_eq!(res.unwrap().name, "Physics");
}

#[test]
fn test_ai_22_rapid_inference_100_files() {
    let config = test_config(Path::new("."));
    let mut rule = test_rule("Log", vec![], vec![]);
    rule.semantic_label = Some("system server logs events messages error".to_string());
    let rules = vec![rule];
    let mut ai = test_ai_engine(&rules).unwrap();

    for i in 0..100 {
        assert!(
            ai.determine_rule(
                &format!("server_system_error_log_{}.log", i),
                &config,
                &rules
            )
            .is_some()
        );
    }
}

#[test]
fn test_ai_23_context_legal() {
    assert!(
        test_ai(
            "non_disclosure_agreement_signed_legal_contract.pdf",
            "legal contracts, agreements, law"
        )
        .is_some()
    );
}

#[test]
fn test_ai_24_context_science() {
    assert!(
        test_ai(
            "neural_network_deep_learning_research_paper.pdf",
            "scientific research, academics, paper"
        )
        .is_some()
    );
}

#[test]
fn test_ai_25_context_software_logs() {
    assert!(
        test_ai(
            "nginx_server_access_log_backup_devops.gz",
            "server logs, access events, devops"
        )
        .is_some()
    );
}

#[test]
fn test_ai_26_context_social_media() {
    assert!(
        test_ai(
            "instagram_marketing_post_caption_engagement.txt",
            "social media, marketing, engagement"
        )
        .is_some()
    );
}

#[test]
fn test_ai_27_edge_filename_path_like() {
    assert!(
        test_ai(
            "Users/Documents/Invoices/financial_billing_invoice.pdf",
            "financial invoice, payment"
        )
        .is_some()
    );
}

#[test]
fn test_ai_28_logic_threshold_cutoff() {
    assert!(
        test_ai(
            "fresh_red_apple_fruit.png",
            "space exploration, mars mission, nasa, astronaut"
        )
        .is_none()
    );
}

#[test]
fn test_ai_29_massive_label_text() {
    let huge_label = "financial banking billing money ".repeat(50);
    assert!(test_ai("monthly_bank_statement.pdf", &huge_label).is_some());
}

#[test]
fn test_ai_30_case_insensitivity() {
    let res1 = test_ai("OFFICIAL_INVOICE.PDF", "financial billing document");
    let res2 = test_ai("official_invoice.pdf", "financial billing document");
    assert_eq!(res1.is_some(), res2.is_some());
}

// ==========================================
// 31-60: AI Performance Benchmarks (Nuance & Stress Test)
// ==========================================

#[test]
fn test_ai_31_ambiguity_bank() {
    // test bank in the context of finance vs nature
    let res = test_ai(
        "river_bank_side_view.jpg",
        "banking, finance, money, investment",
    );
    println!("Benchmark 31 (Bank Ambiguity): {:?}", res);
}

#[test]
fn test_ai_32_ambiguity_apple() {
    // test apple and iphone
    let res = test_ai(
        "apple_iphone_pro_max.jpg",
        "fruits, organic food, healthy snacks",
    );
    println!("Benchmark 32 (Apple Ambiguity): {:?}", res);
}

#[test]
fn test_ai_33_synonyms_resume_cv() {
    // test synonym
    assert!(
        test_ai(
            "my_professional_cv.pdf",
            "resume, job application, work history"
        )
        .is_some()
    );
}

#[test]
fn test_ai_34_jargon_k8s() {
    // kubernetes, cloud infrastructure, container orchestration
    assert!(
        test_ai(
            "deployment_k8s_cluster.yaml",
            "kubernetes, cloud infrastructure, container orchestration"
        )
        .is_some()
    );
}

#[test]
fn test_ai_35_negation_handling() {
    // test negation handling (test whether AI is tricked by the word "not")
    let res = test_ai(
        "not_an_invoice_just_a_letter.pdf",
        "financial invoice, billing",
    );
    println!("Benchmark 35 (Negation): {:?}", res);
}

#[test]
fn test_ai_36_typo_tolerance() {
    // misspelled words
    assert!(test_ai("monly_bank_statment.pdf", "monthly bank statement").is_some());
}

#[test]
fn test_ai_37_abbreviation() {
    // test abbreviation
    assert!(test_ai("stmt_2024_05.pdf", "account statement, financial records").is_some());
}

#[test]
fn test_ai_38_intent_action() {
    // test intent
    assert!(test_ai("things_to_buy_list.txt", "shopping, groceries, commerce").is_some());
}

#[test]
fn test_ai_39_hierarchical_dog_animal() {
    // hierarchical matching from specific level to general level
    assert!(test_ai("golden_retriever_photo.jpg", "animals, pets, dogs").is_some());
}

#[test]
fn test_ai_40_abstract_concept() {
    // abstract concepts
    assert!(
        test_ai(
            "truth_and_justice_essay.docx",
            "philosophy, ethics, abstract ideas"
        )
        .is_some()
    );
}

#[test]
fn test_ai_41_temporal_context() {
    // test temporal context
    assert!(
        test_ai(
            "old_archived_legacy_file_2010.zip",
            "archives, old records, history"
        )
        .is_some()
    );
}

#[test]
fn test_ai_42_verb_intent_install() {
    // test verb intent
    assert!(
        test_ai(
            "install_setup_guide.exe",
            "software installation, setup, installers"
        )
        .is_some()
    );
}

#[test]
fn test_ai_43_meta_data() {
    // test meta data
    assert!(
        test_ai(
            "test_cases_for_ai_model.json",
            "software testing, quality assurance, qa"
        )
        .is_some()
    );
}

#[test]
fn test_ai_44_brand_distinction() {
    let res = test_ai("coca_cola_logo.png", "pepsi, soft drinks, beverage");
    println!("Benchmark 44 (Brand Comparison): {:?}", res);
}

#[test]
fn test_ai_45_location_context() {
    assert!(
        test_ai(
            "london_bridge_vacation.jpg",
            "travel, geography, united kingdom"
        )
        .is_some()
    );
}

#[test]
fn test_ai_46_material_properties() {
    assert!(test_ai("metallic_surface_texture.png", "materials, metal, textures").is_some());
}

#[test]
fn test_ai_47_weather_conditions() {
    assert!(
        test_ai(
            "heavy_thunderstorm_warning.txt",
            "weather, meteorology, atmosphere"
        )
        .is_some()
    );
}

#[test]
fn test_ai_48_professional_events() {
    assert!(
        test_ai(
            "meeting_minutes_board_of_directors.docx",
            "business meetings, corporate events"
        )
        .is_some()
    );
}

#[test]
fn test_ai_49_slang_usage() {
    // ทดสอบ Slang (โมเดล BERT มักจะเข้าใจเพราะเทรนจากเน็ต)
    assert!(test_ai("sick_new_beats_drop.mp3", "cool music, hiphop, trendy").is_some());
}

#[test]
fn test_ai_50_confusing_extension() {
    // ชื่อไฟล์บอกว่าเป็นรูป แต่ในชื่อบอกว่าเป็นข้อความ
    let res = test_ai(
        "text_document_inside_an_image.jpg",
        "plain text files, documents",
    );
    println!("Benchmark 50 (Confusing Ext): {:?}", res);
}

#[test]
fn test_ai_51_security_privacy() {
    assert!(
        test_ai(
            "leaked_passwords_database.sql",
            "security breach, privacy, sensitive data"
        )
        .is_some()
    );
}

#[test]
fn test_ai_52_programming_languages() {
    assert!(
        test_ai(
            "main_function_in_rust_lang.rs",
            "programming, rust, software development"
        )
        .is_some()
    );
}

#[test]
fn test_ai_53_cooking_nuance() {
    assert!(
        test_ai(
            "deep_fried_chicken_wings.jpg",
            "cooking methods, fried food, recipes"
        )
        .is_some()
    );
}

#[test]
fn test_ai_54_file_state_final() {
    assert!(
        test_ai(
            "report_final_final_v2_updated.pdf",
            "completed works, final versions"
        )
        .is_some()
    );
}

#[test]
fn test_ai_55_emotional_sentiment() {
    assert!(
        test_ai(
            "happy_birthday_party_memories.mp4",
            "emotions, joy, celebrations"
        )
        .is_some()
    );
}

#[test]
fn test_ai_56_redundant_emphasis() {
    assert!(
        test_ai(
            "extremely_urgent_important_invoice.pdf",
            "financial billing, invoices"
        )
        .is_some()
    );
}

#[test]
fn test_ai_57_compound_nouns() {
    assert!(
        test_ai(
            "travel_insurance_policy_details.pdf",
            "insurance, travel protection"
        )
        .is_some()
    );
}

#[test]
fn test_ai_58_mathematical_context() {
    assert!(
        test_ai(
            "pi_calculation_algorithm.py",
            "mathematics, science, formulas"
        )
        .is_some()
    );
}

#[test]
fn test_ai_59_empty_signal() {
    // ไฟล์ที่แทบไม่มีชื่อเลย (มีแต่นามสกุล) - ไม่ควรจะ match กับอะไร
    assert!(test_ai(".gitignore", "important legal documents").is_none());
}

#[test]
fn test_ai_60_multilingual_hint() {
    // ทดสอบคำทับศัพท์ (ถ้าโมเดลพอจะรู้)
    let res = test_ai("sushi_menu_tokyo.pdf", "japanese food, seafood, restaurant");
    println!("Benchmark 60 (Multilingual): {:?}", res);
}
