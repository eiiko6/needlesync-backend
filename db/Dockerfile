FROM postgres:15

# Set environment variables
ENV POSTGRES_USER=needlesync
ENV POSTGRES_PASSWORD=secret
ENV POSTGRES_DB=needlesync

# Copy the initialization SQL script
COPY init.sql /docker-entrypoint-initdb.d/
# COPY mock_data.sql /docker-entrypoint-initdb.d/

# Expose port 5432
EXPOSE 5432
