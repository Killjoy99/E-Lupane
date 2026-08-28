import entweni_common


# Initialize logging (pass None to use RUST_LOG from .env, or specify level)
entweni_common.init_logging(None)

def main():
    entweni_common.log_info("Application started...")

if __name__ == "__main__":
    main()