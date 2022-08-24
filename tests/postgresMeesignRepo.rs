// use meesign_server::PostgresMeesignRepo;

// use dotenv::dotenv;

// // TODO: only once
// fn init() -> PostgresMeesignRepo {
//     dotenv().ok();
//     PostgresMeesignRepo::init().unwrap()
// }

// #[test]
// fn create_device() {
//     let db = init();
//     let device_id = vec![1, 2, 3];
//     let device = db.create_device(&device_id, "iphone");
//     assert_eq!(Some(device), db.get_device(&device_id));
// }

// #[test]
// fn create_duplicate_device() {
//     let db = init();
//     let device_id = vec![3, 2, 1];
//     db.create_device(&device_id, "iphone");
//     // TODO
// }
