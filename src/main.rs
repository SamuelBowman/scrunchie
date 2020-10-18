use std::env;
use std::path::Path;
use indicatif::{ ProgressBar, ProgressIterator, ProgressStyle };
use image;
use image::RgbImage;

#[derive(Clone, Debug, PartialEq)]
struct Position(u32, u32);

#[derive(Clone, Debug)]
struct Seam {
    path: Vec<Position>,
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
        bottom_up.push(Seam{ path: vec![Position(x, 0)], cost: energies[x as usize] });
    }

    // Recursive case
    for y in 1..img.height() {
        for x in 0..img.width() {
            let mut best_seam = &Seam{ path: Vec::new(), cost: u32::MAX };

            for position in get_bottom_up_neighbors(img, x, y) {
                let seam = &bottom_up[(position.0 + position.1 * img.width()) as usize];
                if seam.cost < best_seam.cost {
                    best_seam = seam;
                }
            }

            let mut path = best_seam.path.clone(); // inefficient
            path.push(Position(x, y));
            let cost = energies[(x + y * img.width()) as usize] + best_seam.cost;
            bottom_up.push(Seam{ path, cost });
        }
    }

    bottom_up
}

fn determine_best_seam(img: &RgbImage, bottom_up: &Vec<Seam>) -> Seam {
    // Return Seam with lowest cost
    let seams = &bottom_up[((img.width() * (img.height() - 1)) as usize)..];
    seams.iter().min_by_key(|seam| seam.cost).unwrap().clone()
}

fn cut_seam(old_img: RgbImage, bottom_up: &Vec<Seam>) -> RgbImage {
    // Determine best seam
    let best_seam = determine_best_seam(&old_img, &bottom_up);

    // Create new image
    let mut new_img: RgbImage = RgbImage::new(old_img.width() - 1, old_img.height());
    for (y, posn_to_remove) in (0..old_img.height()).zip(best_seam.path.clone()) {
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
    let args: Vec<String> = env::args().collect();
    println!("arguments: {:?}", args);
    let input_file = &args[1];
    let output_file = &args[2];
    // TODO error handling

    // Load image
    let mut img = image::open(Path::new(input_file)).unwrap().into_rgb();
    println!("dimensions: {:?}", img.dimensions());

    // Seam carving
    let columns_to_carve = img.width() * 2 / 3;
    let progress_bar = ProgressBar::new(columns_to_carve.into());
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed}] [{bar:40}] {pos}/{len} ({eta})")
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
        assert_eq!(seam.path.len(), 3);
        assert_eq!(seam.cost, 0);
        assert_eq!(seam.path[0], Position(2, 0));
        assert_eq!(seam.path[1], Position(2, 1));
        assert_eq!(seam.path[2], Position(2, 2));
    }
}
