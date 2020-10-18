use std::path::Path;
use clap::{ Arg, app_from_crate, crate_authors, crate_description, crate_name, crate_version, value_t_or_exit };
use indicatif::{ ProgressBar, ProgressIterator, ProgressStyle };
use image;
use image::RgbImage;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(u32, u32);

#[derive(Clone, Copy, Debug)]
struct Seam {
    posn: Position,
    prev_posn: Option<Position>,
    cost: u32,
}

fn get_von_neumann_neighbors(img: &RgbImage, x: u32, y: u32) -> Vec<Position> {
    let mut neighbors: Vec<Position> = Vec::new();

    if x == 0 {
        neighbors.push(Position(x + 1, y));
    } else if x == img.width() - 1 {
        neighbors.push(Position(x - 1, y));
    } else {
        neighbors.push(Position(x - 1, y));
        neighbors.push(Position(x + 1, y));
    }

    if y == 0 {
        neighbors.push(Position(x, y + 1));
    } else if y == img.height() - 1 {
        neighbors.push(Position(x, y - 1));
    } else {
        neighbors.push(Position(x, y + 1));
        neighbors.push(Position(x, y - 1));
    }

    neighbors
}

fn calculate_energy(img: &RgbImage, x: u32, y: u32) -> u32 {
    let mut energy: u32 = 0;
    let center_pixel = img.get_pixel(x, y);

    for neighbor_position in get_von_neumann_neighbors(img, x, y) {
        let neighbor_pixel = img.get_pixel(neighbor_position.0, neighbor_position.1);
        energy += (neighbor_pixel[0] as i32 - center_pixel[0] as i32).pow(2) as u32;
        energy += (neighbor_pixel[1] as i32 - center_pixel[1] as i32).pow(2) as u32;
        energy += (neighbor_pixel[2] as i32 - center_pixel[2] as i32).pow(2) as u32;
    }

    energy
}

fn generate_energies_vector(img: &RgbImage) -> Vec<u32> {
    let mut energies: Vec<u32> = Vec::new();

    for y in 0..img.height() {
        for x in 0..img.width() {
            energies.push(calculate_energy(img, x, y));
        }
    }

    energies
}

fn get_bottom_up_neighbors(img: &RgbImage, x: u32, y: u32) -> Vec<Position> {
    let mut neighbors: Vec<Position> = Vec::new();

    if y == 0 {
        return neighbors;
    }
    neighbors.push(Position(x, y - 1));
    if x == 0 {
        neighbors.push(Position(x + 1, y - 1));
        return neighbors;
    } else if x == img.width() - 1 {
        neighbors.push(Position(x - 1, y - 1));
        return neighbors;
    } else {
        neighbors.push(Position(x + 1, y - 1));
        neighbors.push(Position(x - 1, y - 1));
        return neighbors;
    }
}

fn generate_bottom_up_vector(img: &RgbImage, energies: &Vec<u32>) -> Vec<Seam> {
    let mut bottom_up: Vec<Seam> = Vec::new();

    // Base case
    for x in 0..img.width() {
        bottom_up.push(Seam{ posn: Position(x, 0), prev_posn: None, cost: energies[x as usize] });
    }

    // Recursive case
    for y in 1..img.height() {
        for x in 0..img.width() {
            let prev_posn = *get_bottom_up_neighbors(img, x, y).iter().min_by_key(|posn| bottom_up[(posn.0 + posn.1 * img.width()) as usize].cost).unwrap();
            let cost = energies[(x + y * img.width()) as usize] + bottom_up[(prev_posn.0 + prev_posn.1 * img.width()) as usize].cost;
            bottom_up.push(Seam{ posn: Position(x, y), prev_posn: Some(prev_posn), cost });
        }
    }

    bottom_up
}

fn determine_best_seam<'a>(img: &RgbImage, bottom_up: &'a Vec<Seam>) -> &'a Seam {
    // Return Seam with lowest cost
    bottom_up[((img.width() * (img.height() - 1)) as usize)..].iter().min_by_key(|seam| seam.cost).unwrap()
}

fn seam_to_position_vector(img: &RgbImage, bottom_up: &Vec<Seam>, initial_seam: &Seam) -> Vec<Position> {
    let mut seam = initial_seam;
    let mut posn_vector = Vec::new();
    posn_vector.push(seam.posn);
    while let Some(prev_posn) = seam.prev_posn {
        posn_vector.push(prev_posn);
        seam = &bottom_up[(prev_posn.0 + prev_posn.1 * img.width()) as usize];
    }
    posn_vector.reverse();
    posn_vector
}

fn cut_seam(old_img: RgbImage, bottom_up: &Vec<Seam>) -> RgbImage {
    // Determine best seam
    let seam = determine_best_seam(&old_img, &bottom_up);
    let posns_to_remove = seam_to_position_vector(&old_img, &bottom_up, &seam);

    // Create new image
    let mut new_img: RgbImage = RgbImage::new(old_img.width() - 1, old_img.height());
    for (y, posn_to_remove) in (0..old_img.height()).zip(posns_to_remove) {
        let mut new_x = 0;
        for old_x in 0..old_img.width() {
            if !(old_x == posn_to_remove.0) {
                new_img.put_pixel(new_x, y, *old_img.get_pixel(old_x, y));
                new_x += 1;
            }
        }
    }

    new_img
}

fn main() {
    // Print banner
    println!(r"                                  _     _      ");
    println!(r"   ___  ___ _ __ _   _ _ __   ___| |__ (_) ___ ");
    println!(r"  / __|/ __| '__| | | | '_ \ / __| '_ \| |/ _ \");
    println!(r"  \__ \ (__| |  | |_| | | | | (__| | | | |  __/ by Sam");
    println!(r"  |___/\___|_|   \__,_|_| |_|\___|_| |_|_|\___| Bowman");
    println!(r"________________________________________________________");
    println!();

    // Parse arguments
    let matches = app_from_crate!()
        .arg(Arg::with_name("percentage")
            .short("p")
            .default_value("66")
            .help("Percentage of image to scrunch"))
        .arg(Arg::with_name("input_file")
            .required(true)
            .help("Input image path"))
        .arg(Arg::with_name("output_file")
            .required(true)
            .help("Output image path"))
        .get_matches();
    let input_file = matches.value_of("input_file").unwrap();
    let output_file = matches.value_of("output_file").unwrap();
    let percentage = value_t_or_exit!(matches.value_of("percentage"), u32);

    // Load image and print details
    let mut img = image::open(Path::new(input_file)).unwrap().into_rgb();
    println!("Source Resolution: {} x {} ({} pixels)", img.width(), img.height(), img.width() * img.height());
    let columns_to_carve = img.width() * percentage / 100;
    println!("Columns To Carve: {} ({}%)", columns_to_carve, percentage);
    println!();

    // Seam carving
    let progress_bar = ProgressBar::new(columns_to_carve as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40}] {pos}/{len} ")
        .progress_chars("#>-"));
    for _ in (0..columns_to_carve).progress_with(progress_bar) {
        let energies = generate_energies_vector(&img);
        let bottom_up = generate_bottom_up_vector(&img, &energies);
        img = cut_seam(img, &bottom_up);
    }

    // Save image
    img.save(Path::new(output_file)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    fn open_test_image() -> RgbImage {
        image::open(Path::new("/home/sam/projects/classes/H343/h343/L7/balloon-sky.jpg")).unwrap().into_rgb()
    }

    #[test]
    fn test_get_von_neumann_neighbors() {
        let img = open_test_image();

        let neighbors = get_von_neumann_neighbors(&img, 0, 0);
        assert!(neighbors.contains(&Position(0, 1)));
        assert!(neighbors.contains(&Position(1, 0)));
        assert_eq!(neighbors.len(), 2);

        let neighbors = get_von_neumann_neighbors(&img, img.width() / 2, 0);
        assert!(neighbors.contains(&Position(img.width() / 2 - 1, 0)));
        assert!(neighbors.contains(&Position(img.width() / 2 + 1, 0)));
        assert!(neighbors.contains(&Position(img.width() / 2, 1)));
        assert_eq!(neighbors.len(), 3);

        let neighbors = get_von_neumann_neighbors(&img, img.width() - 1, 0);
        assert!(neighbors.contains(&Position(img.width() - 2, 0)));
        assert!(neighbors.contains(&Position(img.width() - 1, 1)));
        assert_eq!(neighbors.len(), 2);

        let neighbors = get_von_neumann_neighbors(&img, 0, img.height() / 2);
        assert!(neighbors.contains(&Position(0, img.height() / 2 - 1)));
        assert!(neighbors.contains(&Position(0, img.height() / 2 + 1)));
        assert!(neighbors.contains(&Position(1, img.height() / 2)));
        assert_eq!(neighbors.len(), 3);

        let neighbors = get_von_neumann_neighbors(&img, img.width() / 2, img.height() / 2);
        assert!(neighbors.contains(&Position(img.width() / 2, img.height() / 2 - 1)));
        assert!(neighbors.contains(&Position(img.width() / 2, img.height() / 2 + 1)));
        assert!(neighbors.contains(&Position(img.width() / 2 - 1, img.height() / 2)));
        assert!(neighbors.contains(&Position(img.width() / 2 + 1, img.height() / 2)));
        assert_eq!(neighbors.len(), 4);

        let neighbors = get_von_neumann_neighbors(&img, img.width() - 1, img.height() / 2);
        assert!(neighbors.contains(&Position(img.width() - 2, img.height() / 2)));
        assert!(neighbors.contains(&Position(img.width() - 1, img.height() / 2 - 1)));
        assert!(neighbors.contains(&Position(img.width() - 1, img.height() / 2 + 1)));
        assert_eq!(neighbors.len(), 3);

        let neighbors = get_von_neumann_neighbors(&img, 0, img.height() - 1);
        assert!(neighbors.contains(&Position(0, img.height() - 2)));
        assert!(neighbors.contains(&Position(1, img.height() - 1)));
        assert_eq!(neighbors.len(), 2);

        let neighbors = get_von_neumann_neighbors(&img, img.width() / 2, img.height() - 1);
        assert!(neighbors.contains(&Position(img.width() / 2, img.height() - 2)));
        assert!(neighbors.contains(&Position(img.width() / 2 - 1, img.height() - 1)));
        assert!(neighbors.contains(&Position(img.width() / 2 + 1, img.height() - 1)));
        assert_eq!(neighbors.len(), 3);

        let neighbors = get_von_neumann_neighbors(&img, img.width() - 1, img.height() - 1);
        assert!(neighbors.contains(&Position(img.width() - 1, img.height() - 2)));
        assert!(neighbors.contains(&Position(img.width() - 1, img.height() - 2)));
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_get_bottom_up_neighbors() {
        // TODO
    }

    #[test]
    fn test_calculate_energy() {
        let img = open_test_image();
        assert_eq!(calculate_energy(&img, 748, 28), 20);
        assert_eq!(calculate_energy(&img, 406, 59), 84);
        assert_eq!(calculate_energy(&img, 462, 92), 39);
        assert_eq!(calculate_energy(&img, 332, 101), 0);
        assert_eq!(calculate_energy(&img, 602, 237), 96);
        assert_eq!(calculate_energy(&img, 34, 387), 7);
        assert_eq!(calculate_energy(&img, 673, 394), 0);
        assert_eq!(calculate_energy(&img, 213, 397), 6);
        assert_eq!(calculate_energy(&img, 63, 442), 84);
        assert_eq!(calculate_energy(&img, 388, 510), 16);
        assert_eq!(calculate_energy(&img, 899, 535), 0);
        assert_eq!(calculate_energy(&img, 689, 546), 27);
        assert_eq!(calculate_energy(&img, 359, 599), 26);
        assert_eq!(calculate_energy(&img, 4, 629), 23);
        assert_eq!(calculate_energy(&img, 53, 673), 0);
    }

    #[test]
    fn test_determine_best_seam() {
        let mut img = image::RgbImage::new(5, 3);
        let red = Rgb([255, 0 , 0]);
        let blue = Rgb([0, 0, 255]);
        img.put_pixel(0, 0, red);
        img.put_pixel(1, 0, blue);
        img.put_pixel(2, 0, blue);
        img.put_pixel(3, 0, blue);
        img.put_pixel(4, 0, red);
        img.put_pixel(0, 1, blue);
        img.put_pixel(1, 1, blue);
        img.put_pixel(2, 1, blue);
        img.put_pixel(3, 1, blue);
        img.put_pixel(4, 1, blue);
        img.put_pixel(0, 2, red);
        img.put_pixel(1, 2, blue);
        img.put_pixel(2, 2, blue);
        img.put_pixel(3, 2, blue);
        img.put_pixel(4, 2, red);

        let energies = generate_energies_vector(&img);
        let bottom_up = generate_bottom_up_vector(&img, &energies);
        let seam = determine_best_seam(&img, &bottom_up);
        let posns_to_remove = seam_to_position_vector(&img, &bottom_up, &seam);
        assert_eq!(posns_to_remove.len(), 3);
        assert_eq!(seam.cost, 0);
        assert_eq!(posns_to_remove[0], Position(2, 0));
        assert_eq!(posns_to_remove[1], Position(2, 1));
        assert_eq!(posns_to_remove[2], Position(2, 2));
    }
}
