-- MySQL Workbench Forward Engineering

SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0;
SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0;
SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION';

-- -----------------------------------------------------
-- Schema mydb
-- -----------------------------------------------------
-- -----------------------------------------------------
-- Schema ctfpkhk
-- -----------------------------------------------------

-- -----------------------------------------------------
-- Schema ctfpkhk
-- -----------------------------------------------------
CREATE SCHEMA IF NOT EXISTS `ctfpkhk` DEFAULT CHARACTER SET utf8mb3 ;
USE `ctfpkhk` ;

-- -----------------------------------------------------
-- Table `ctfpkhk`.`attachments`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`attachments` (
  `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `challenge_id` INT UNSIGNED NULL DEFAULT NULL,
  `event_id` INT UNSIGNED NULL DEFAULT NULL,
  `file_name` VARCHAR(90) NOT NULL,
  `file_blob` MEDIUMBLOB NOT NULL,
  `file_type` VARCHAR(20) NOT NULL,
  `mime_type` VARCHAR(45) NULL DEFAULT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
AUTO_INCREMENT = 10
DEFAULT CHARACTER SET = utf8mb3;

CREATE INDEX `fk_attachments_challenges_idx` ON `ctfpkhk`.`attachments` (`challenge_id` ASC) VISIBLE;

CREATE INDEX `fk_attachments_events1_idx` ON `ctfpkhk`.`attachments` (`event_id` ASC) VISIBLE;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`events`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`events` (
  `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `name` VARCHAR(45) NOT NULL,
  `description` TEXT NULL DEFAULT NULL,
  `start_date` TIMESTAMP NOT NULL,
  `end_date` TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
AUTO_INCREMENT = 2
DEFAULT CHARACTER SET = utf8mb3;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`challenges`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`challenges` (
  `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `event_id` INT UNSIGNED NOT NULL,
  `name` VARCHAR(45) NOT NULL,
  `description` TEXT NULL DEFAULT NULL,
  `category` VARCHAR(45) NULL DEFAULT NULL,
  `difficulty` TINYINT NOT NULL,
  `points` INT UNSIGNED NOT NULL,
  `flag_hash` VARCHAR(100) NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `fk_challenges_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`))
ENGINE = InnoDB
AUTO_INCREMENT = 21
DEFAULT CHARACTER SET = utf8mb3;

CREATE INDEX `fk_challenges_events1_idx` ON `ctfpkhk`.`challenges` (`event_id` ASC) VISIBLE;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`users`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`users` (
  `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `avatar` MEDIUMBLOB NULL DEFAULT NULL,
  `username` VARCHAR(40) NOT NULL,
  `email` VARCHAR(90) NOT NULL,
  `pw_hash` VARCHAR(100) NOT NULL,
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `last_active_at` TIMESTAMP NOT NULL,
  `role` VARCHAR(14) NOT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
AUTO_INCREMENT = 4
DEFAULT CHARACTER SET = utf8mb3;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`submissions`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`submissions` (
  `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `challenge_id` INT UNSIGNED NOT NULL,
  `event_id` INT UNSIGNED NOT NULL,
  `user_id` INT UNSIGNED NOT NULL,
  `points` INT UNSIGNED NOT NULL,
  `solved_at` TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `fk_leaderboard_challenges1`
    FOREIGN KEY (`challenge_id`)
    REFERENCES `ctfpkhk`.`challenges` (`id`),
  CONSTRAINT `fk_leaderboard_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`),
  CONSTRAINT `fk_leaderboard_users1`
    FOREIGN KEY (`user_id`)
    REFERENCES `ctfpkhk`.`users` (`id`))
ENGINE = InnoDB
AUTO_INCREMENT = 16
DEFAULT CHARACTER SET = utf8mb3;

CREATE INDEX `fk_leaderboard_users1_idx` ON `ctfpkhk`.`submissions` (`user_id` ASC) VISIBLE;

CREATE INDEX `fk_leaderboard_events1_idx` ON `ctfpkhk`.`submissions` (`event_id` ASC) VISIBLE;

CREATE INDEX `fk_leaderboard_challenges1_idx` ON `ctfpkhk`.`submissions` (`challenge_id` ASC) VISIBLE;


SET SQL_MODE=@OLD_SQL_MODE;
SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS;
SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS;
